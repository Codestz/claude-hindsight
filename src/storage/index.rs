//! Session indexing with SQLite
//!
//! Maintains a fast SQLite index of all discovered sessions for quick lookups.

use crate::error::{HindsightError, Result};
use crate::storage::SessionFile;
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// SQLite-based session index for fast lookups
pub struct SessionIndex {
    conn: Connection,
}

impl SessionIndex {
    /// Create or open the session index database
    ///
    /// The database is stored at `~/.config/hindsight/sessions.db`
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            HindsightError::Config("Could not determine config directory".to_string())
        })?;

        let hindsight_dir = config_dir.join("claude-hindsight");
        std::fs::create_dir_all(&hindsight_dir)?;

        let db_path = hindsight_dir.join("sessions.db");
        let conn = Connection::open(db_path)?;

        let mut index = SessionIndex { conn };
        index.initialize_schema()?;

        Ok(index)
    }

    /// Create a new in-memory session index for testing
    #[cfg(test)]
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut index = SessionIndex { conn };
        index.initialize_schema()?;
        Ok(index)
    }

    /// Initialize the database schema
    fn initialize_schema(&mut self) -> Result<()> {
        // Check schema version
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if version == 0 {
            // Fresh database: create schema v6 with all tables
            self.conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    project_name TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    file_size INTEGER NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT 0,
                    modified_at INTEGER NOT NULL,
                    has_subagents INTEGER NOT NULL,
                    indexed_at INTEGER NOT NULL,
                    model TEXT,
                    error_count INTEGER NOT NULL DEFAULT 0,
                    first_message TEXT,
                    source_dir TEXT NOT NULL DEFAULT '',
                    subagent_models TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_project_name ON sessions(project_name);
                CREATE INDEX IF NOT EXISTS idx_modified_at ON sessions(modified_at DESC);
                CREATE INDEX IF NOT EXISTS idx_has_subagents ON sessions(has_subagents);

                CREATE TABLE IF NOT EXISTS tool_usage (
                    session_id TEXT NOT NULL,
                    tool_name TEXT NOT NULL,
                    usage_count INTEGER NOT NULL,
                    PRIMARY KEY (session_id, tool_name),
                    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_tool_name ON tool_usage(tool_name);
                CREATE INDEX IF NOT EXISTS idx_usage_count ON tool_usage(usage_count DESC);

                CREATE TABLE IF NOT EXISTS file_usage (
                    session_id TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    access_count INTEGER NOT NULL,
                    PRIMARY KEY (session_id, file_path),
                    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_file_path ON file_usage(file_path);
                CREATE INDEX IF NOT EXISTS idx_file_access_count ON file_usage(access_count DESC);

                CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
                    session_id UNINDEXED,
                    searchable_text,
                    tokenize='porter ascii'
                );
                "#,
            )?;
            self.conn.execute("PRAGMA user_version = 8", [])?;
        } else {
            // Incremental migrations
            if version < 4 {
                // v1/v2/v3 → v4: add missing columns
                for stmt in &[
                    "ALTER TABLE sessions ADD COLUMN total_tokens INTEGER NOT NULL DEFAULT 0",
                    "ALTER TABLE sessions ADD COLUMN estimated_cost REAL NOT NULL DEFAULT 0.0",
                    "ALTER TABLE sessions ADD COLUMN model TEXT",
                    "ALTER TABLE sessions ADD COLUMN error_count INTEGER NOT NULL DEFAULT 0",
                    "ALTER TABLE sessions ADD COLUMN first_message TEXT",
                    "ALTER TABLE sessions ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0",
                ] {
                    let _ = self.conn.execute(stmt, []);
                }
                let _ = self.conn.execute_batch(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(session_id UNINDEXED, searchable_text, tokenize='porter ascii');"
                );
                self.conn.execute("PRAGMA user_version = 4", [])?;
            }

            if version < 5 {
                // v4 → v5: add file_usage table
                self.conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS file_usage (
                        session_id TEXT NOT NULL,
                        file_path TEXT NOT NULL,
                        access_count INTEGER NOT NULL,
                        PRIMARY KEY (session_id, file_path),
                        FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
                    );
                    CREATE INDEX IF NOT EXISTS idx_file_path ON file_usage(file_path);
                    CREATE INDEX IF NOT EXISTS idx_file_access_count ON file_usage(access_count DESC);
                    "#,
                )?;
                self.conn.execute("PRAGMA user_version = 5", [])?;
            }

            if version < 6 {
                // v5 → v6: add source_dir and subagent_models columns
                let _ = self.conn.execute(
                    "ALTER TABLE sessions ADD COLUMN source_dir TEXT NOT NULL DEFAULT ''",
                    [],
                );
                let _ = self.conn.execute(
                    "ALTER TABLE sessions ADD COLUMN subagent_models TEXT",
                    [],
                );
                self.conn.execute("PRAGMA user_version = 6", [])?;
            }

            if version < 7 {
                // v6 → v7: added token breakdown columns (now unused but harmless)
                for stmt in &[
                    "ALTER TABLE sessions ADD COLUMN input_tokens INTEGER NOT NULL DEFAULT 0",
                    "ALTER TABLE sessions ADD COLUMN output_tokens INTEGER NOT NULL DEFAULT 0",
                    "ALTER TABLE sessions ADD COLUMN cache_creation_tokens INTEGER NOT NULL DEFAULT 0",
                    "ALTER TABLE sessions ADD COLUMN cache_read_tokens INTEGER NOT NULL DEFAULT 0",
                ] {
                    let _ = self.conn.execute(stmt, []);
                }
                self.conn.execute("PRAGMA user_version = 7", [])?;
            }

            if version < 8 {
                // v7 → v8: token/cost columns removed from application layer.
                // Orphaned columns (total_tokens, estimated_cost, input_tokens, etc.)
                // are left in place — SQLite handles unused columns gracefully.
                self.conn.execute("PRAGMA user_version = 8", [])?;
            }
        }

        Ok(())
    }

    /// Index a single session file
    pub fn index_session(&mut self, session: &SessionFile) -> Result<()> {
        use std::collections::HashMap;

        let indexed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Try to parse session for rich analytics; fall back to defaults on failure
        let (model, error_count, first_message, tool_counts, file_counts, created_at) =
            if let Ok(parsed) = crate::parser::parse_session(&session.path) {
                let analytics = crate::analyzer::SessionAnalytics::from_session(&parsed);

                // Earliest node timestamp → session creation time (ms → secs)
                let created_at = parsed
                    .nodes
                    .iter()
                    .find_map(|n| n.timestamp)
                    .map(|ms| ms / 1000)
                    .unwrap_or(session.modified_at);

                // First user message text preview (skip local commands).
                let first_message: Option<String> = parsed
                    .nodes
                    .iter()
                    .filter(|n| n.node_type == "user")
                    .find_map(|n| {
                        let text = n.message.as_ref()?.text_content();
                        let trimmed = text.trim().to_string();
                        if trimmed.is_empty() {
                            return None;
                        }
                        if crate::analyzer::prompt_detect::is_local_command_text(&trimmed) {
                            return None;
                        }
                        if crate::analyzer::prompt_detect::is_trivial_message(&trimmed) {
                            return None;
                        }
                        let preview = trimmed.replace('\n', " ");
                        Some(preview.chars().take(300).collect::<String>())
                    });

                // Build tool counts and file access counts
                let mut tool_counts: HashMap<String, usize> = HashMap::new();
                let mut file_counts: HashMap<String, usize> = HashMap::new();
                for node in &parsed.nodes {
                    if let Some(ref tool_use) = node.tool_use {
                        *tool_counts.entry(tool_use.name.clone()).or_insert(0) += 1;
                        if let Some(path) = file_path_from_input(&tool_use.name, &tool_use.input) {
                            *file_counts.entry(path).or_insert(0) += 1;
                        }
                    }
                    for block in node
                        .message
                        .as_ref()
                        .map(|m| m.content_blocks())
                        .unwrap_or(&[])
                    {
                        if let crate::parser::models::ContentBlock::ToolUse { name, input, .. } =
                            block
                        {
                            *tool_counts.entry(name.clone()).or_insert(0) += 1;
                            if let Some(path) = file_path_from_input(name, input) {
                                *file_counts.entry(path).or_insert(0) += 1;
                            }
                        }
                    }
                }

                (
                    parsed.model.clone(),
                    analytics.error_count as i64,
                    first_message,
                    Some(tool_counts),
                    Some(file_counts),
                    created_at,
                )
            } else {
                (
                    None::<String>,
                    0i64,
                    None::<String>,
                    None,
                    None,
                    session.modified_at,
                )
            };

        // Collect subagent models
        let subagent_sessions = crate::parser::parse_subagents(&session.path);

        let mut sub_models: Vec<String> = subagent_sessions
            .iter()
            .filter_map(|s| s.model.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        sub_models.sort();
        let subagent_models_str: Option<String> = if sub_models.is_empty() {
            None
        } else {
            Some(sub_models.join(","))
        };

        // Skip sessions with no meaningful content (file-history-snapshots, abandoned
        // test files, etc.). Remove from index if previously added.
        if model.is_none() && first_message.is_none() {
            self.conn.execute(
                "DELETE FROM sessions WHERE session_id = ?1",
                params![session.session_id],
            )?;
            return Ok(());
        }

        // Start a transaction for atomic updates
        let tx = self.conn.transaction()?;

        // Insert/update session metadata with analytics
        tx.execute(
            r#"
            INSERT OR REPLACE INTO sessions
            (session_id, project_name, file_path, file_size, created_at, modified_at, has_subagents, indexed_at,
             model, error_count, first_message, source_dir, subagent_models)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                session.session_id,
                session.project_name,
                session.path.to_string_lossy(),
                session.file_size as i64,
                created_at,
                session.modified_at,
                if session.has_subagents { 1 } else { 0 },
                indexed_at,
                model,
                error_count,
                first_message,
                session.source_dir,
                subagent_models_str,
            ],
        )?;

        // Refresh FTS entry (delete-then-insert is the correct FTS5 update pattern)
        tx.execute(
            "DELETE FROM sessions_fts WHERE session_id = ?1",
            params![session.session_id],
        )?;
        tx.execute(
            "INSERT INTO sessions_fts (session_id, searchable_text) VALUES (?1, ?2)",
            params![
                session.session_id,
                format!(
                    "{} {} {}",
                    session.project_name,
                    first_message.as_deref().unwrap_or(""),
                    model.as_deref().unwrap_or(""),
                ),
            ],
        )?;

        // Update tool_usage if we successfully parsed
        if let Some(tool_counts) = tool_counts {
            tx.execute(
                "DELETE FROM tool_usage WHERE session_id = ?1",
                params![session.session_id],
            )?;

            let mut stmt = tx.prepare(
                "INSERT INTO tool_usage (session_id, tool_name, usage_count) VALUES (?1, ?2, ?3)",
            )?;

            for (tool_name, count) in tool_counts {
                stmt.execute(params![session.session_id, tool_name, count as i64])?;
            }
        }

        // Update file_usage if we successfully parsed
        if let Some(file_counts) = file_counts {
            tx.execute(
                "DELETE FROM file_usage WHERE session_id = ?1",
                params![session.session_id],
            )?;

            let mut stmt = tx.prepare(
                "INSERT INTO file_usage (session_id, file_path, access_count) VALUES (?1, ?2, ?3)",
            )?;

            for (file_path, count) in file_counts {
                stmt.execute(params![session.session_id, file_path, count as i64])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Index all discovered sessions
    pub fn index_all(&mut self, sessions: &[SessionFile]) -> Result<usize> {
        let mut count = 0;

        for session in sessions {
            self.index_session(session)?;
            count += 1;
        }

        Ok(count)
    }

    /// Get all sessions, sorted by modification time (newest first)
    pub fn list_sessions(&self) -> Result<Vec<SessionFile>> {
        let mut stmt = self.conn.prepare(
            &format!("SELECT {} FROM sessions ORDER BY modified_at DESC", SESSION_COLS),
        )?;

        let sessions = stmt
            .query_map([], session_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Find sessions by project name (excludes empty sessions)
    pub fn find_by_project(&self, project: &str) -> Result<Vec<SessionFile>> {
        let mut stmt = self.conn.prepare(
            &format!(
                "SELECT {} FROM sessions WHERE project_name = ?1 \
                 AND (first_message IS NOT NULL OR model IS NOT NULL) \
                 ORDER BY modified_at DESC",
                SESSION_COLS
            ),
        )?;

        let sessions = stmt
            .query_map([project], session_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Find a session by ID (exact match or prefix)
    pub fn find_by_id(&self, session_id: &str) -> Result<Option<SessionFile>> {
        // Try exact match first
        let mut stmt = self.conn.prepare(
            &format!("SELECT {} FROM sessions WHERE session_id = ?1", SESSION_COLS),
        )?;

        let result = stmt.query_row([session_id], session_from_row);

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Try prefix match
                let mut stmt = self.conn.prepare(
                    &format!(
                        "SELECT {} FROM sessions WHERE session_id LIKE ?1 || '%' \
                         ORDER BY modified_at DESC LIMIT 1",
                        SESSION_COLS
                    ),
                )?;

                let result = stmt.query_row([session_id], session_from_row);

                match result {
                    Ok(session) => Ok(Some(session)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Get the most recently modified session
    pub fn get_latest(&self) -> Result<Option<SessionFile>> {
        let mut stmt = self.conn.prepare(
            &format!("SELECT {} FROM sessions ORDER BY modified_at DESC LIMIT 1", SESSION_COLS),
        )?;

        let result = stmt.query_row([], session_from_row);

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Remove sessions that no longer exist on disk
    pub fn prune_missing(&mut self) -> Result<usize> {
        let sessions = self.list_sessions()?;
        let mut removed = 0;

        for session in sessions {
            if !session.path.exists() {
                self.conn.execute(
                    "DELETE FROM sessions_fts WHERE session_id = ?1",
                    params![session.session_id],
                )?;
                self.conn.execute(
                    "DELETE FROM sessions WHERE session_id = ?1",
                    params![session.session_id],
                )?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    /// Get total number of indexed sessions
    #[allow(dead_code)]
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;

        Ok(count as usize)
    }

    /// Get all unique project names
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT project_name FROM sessions ORDER BY project_name")?;

        let projects = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Get project statistics
    pub fn get_project_stats(&self, project: &str) -> Result<ProjectStats> {
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*) as session_count,
                SUM(file_size) as total_size,
                MAX(modified_at) as last_activity
             FROM sessions
             WHERE project_name = ?1",
        )?;

        let stats = stmt.query_row([project], |row| {
            Ok(ProjectStats {
                project_name: project.to_string(),
                session_count: row.get::<_, i64>(0)? as usize,
                total_size: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                last_activity: row.get::<_, Option<i64>>(2)?,
            })
        })?;

        Ok(stats)
    }

    /// Get all project statistics
    pub fn get_all_project_stats(&self) -> Result<Vec<ProjectStats>> {
        let projects = self.list_projects()?;
        let mut stats = Vec::new();

        for project in projects {
            stats.push(self.get_project_stats(&project)?);
        }

        // Sort by last activity (most recent first)
        stats.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(stats)
    }

    /// Get global analytics across all sessions
    pub fn get_global_analytics(&self) -> Result<GlobalAnalytics> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let one_week_ago = now - (7 * 24 * 60 * 60);
        let today_start = now - (now % (24 * 60 * 60));

        // Total sessions, size, errors
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(file_size), COALESCE(SUM(error_count),0) FROM sessions",
        )?;

        let (total_sessions, total_size, total_errors) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                row.get::<_, i64>(2)? as usize,
            ))
        })?;

        // Sessions this week
        let sessions_this_week: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE modified_at >= ?1",
            [one_week_ago],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Sessions today
        let sessions_today: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE modified_at >= ?1",
            [today_start],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Total projects
        let total_projects = self.list_projects()?.len();

        // Subagent count
        let subagent_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE has_subagents = 1",
            [],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Average session size
        let avg_session_size = if total_sessions > 0 {
            total_size / total_sessions as u64
        } else {
            0
        };

        // Most active project
        let most_active_project = self
            .conn
            .query_row(
                "SELECT project_name FROM sessions ORDER BY modified_at DESC LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok();

        // Top tools - parse recent sessions to extract tool usage
        let top_tools = self.get_top_tools(100)?;

        Ok(GlobalAnalytics {
            total_sessions,
            sessions_this_week,
            sessions_today,
            total_size,
            total_projects,
            subagent_count,
            avg_session_size,
            most_active_project,
            top_tools,
            total_errors,
        })
    }

    /// Extract top tools from recent sessions (using indexed tool_usage table)
    fn get_top_tools(&self, _session_limit: usize) -> Result<Vec<(String, usize)>> {
        // Query aggregates tool usage across all sessions (no file parsing!)
        let mut stmt = self.conn.prepare(
            r#"
            SELECT tool_name, SUM(usage_count) as total_count
            FROM tool_usage
            GROUP BY tool_name
            ORDER BY total_count DESC
            LIMIT 30
            "#,
        )?;

        let top_tools = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(top_tools)
    }

    /// Get project-specific analytics
    pub fn get_project_analytics(&self, project: &str) -> Result<ProjectAnalytics> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let one_week_ago = now - (7 * 24 * 60 * 60);
        let today_start = now - (now % (24 * 60 * 60));

        // Total sessions, size, errors for this project
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(file_size), COALESCE(SUM(error_count),0) \
             FROM sessions WHERE project_name = ?1",
        )?;

        let (total_sessions, total_size, total_errors) = stmt.query_row([project], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                row.get::<_, i64>(2)? as usize,
            ))
        })?;

        // Sessions this week
        let sessions_this_week: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE project_name = ?1 AND modified_at >= ?2",
            [project, &one_week_ago.to_string()],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Sessions today
        let sessions_today: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE project_name = ?1 AND modified_at >= ?2",
            [project, &today_start.to_string()],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Subagent count
        let subagent_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE project_name = ?1 AND has_subagents = 1",
            [project],
            |row| row.get::<_, i64>(0).map(|c| c as usize),
        )?;

        // Average session size
        let avg_session_size = if total_sessions > 0 {
            total_size / total_sessions as u64
        } else {
            0
        };

        // Last activity
        let last_activity = self.conn.query_row(
            "SELECT modified_at FROM sessions WHERE project_name = ?1 ORDER BY modified_at DESC LIMIT 1",
            [project],
            |row| row.get::<_, i64>(0),
        ).ok();

        // Top tools for this project
        let top_tools = self.get_top_tools_for_project(project, 50)?;

        Ok(ProjectAnalytics {
            project_name: project.to_string(),
            total_sessions,
            sessions_this_week,
            sessions_today,
            total_size,
            subagent_count,
            avg_session_size,
            top_tools,
            last_activity,
            total_errors,
        })
    }

    /// Get top tools for a specific project
    fn get_top_tools_for_project(
        &self,
        project: &str,
        _session_limit: usize,
    ) -> Result<Vec<(String, usize)>> {
        // Query aggregates tool usage for this project (no file parsing!)
        let mut stmt = self.conn.prepare(
            r#"
            SELECT t.tool_name, SUM(t.usage_count) as total_count
            FROM tool_usage t
            JOIN sessions s ON t.session_id = s.session_id
            WHERE s.project_name = ?1
            GROUP BY t.tool_name
            ORDER BY total_count DESC
            LIMIT 30
            "#,
        )?;

        let top_tools = stmt
            .query_map([project], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(top_tools)
    }

    /// Get top accessed files globally
    pub fn get_top_files(&self, limit: usize) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT file_path, SUM(access_count) as total
            FROM file_usage
            GROUP BY file_path
            ORDER BY total DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Get top accessed files for a specific project
    pub fn get_top_files_for_project(
        &self,
        project: &str,
        limit: usize,
    ) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT f.file_path, SUM(f.access_count) as total
            FROM file_usage f
            JOIN sessions s ON f.session_id = s.session_id
            WHERE s.project_name = ?1
            GROUP BY f.file_path
            ORDER BY total DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt
            .query_map(params![project, limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Get sessions that used any of the specified tools
    #[allow(dead_code)]
    pub fn find_by_tools(&self, tool_names: &[String]) -> Result<Vec<SessionFile>> {
        if tool_names.is_empty() {
            return Ok(Vec::new());
        }

        // Build SQL query with placeholders for tool names
        let placeholders = tool_names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT DISTINCT {} FROM sessions s \
             JOIN tool_usage t ON s.session_id = t.session_id \
             WHERE t.tool_name IN ({}) \
             ORDER BY s.modified_at DESC",
            SESSION_COLS_PREFIXED, placeholders
        );

        let mut stmt = self.conn.prepare(&query)?;

        let params: Vec<&dyn rusqlite::ToSql> = tool_names
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let sessions = stmt
            .query_map(params.as_slice(), session_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get session counts per day for the last `days` days (for sparkline)
    /// Returns a Vec of `days` counts, oldest first
    pub fn get_daily_session_counts(&self, days: usize) -> Result<Vec<u64>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let day_secs = 24 * 60 * 60i64;
        // today's bucket start (floor to day boundary)
        let today = now - (now % day_secs);

        let mut counts = vec![0u64; days];

        let window_start = today - ((days as i64 - 1) * day_secs);

        let mut stmt = self.conn.prepare(
            "SELECT modified_at FROM sessions WHERE modified_at >= ?1 ORDER BY modified_at",
        )?;

        let rows = stmt.query_map([window_start], |row| row.get::<_, i64>(0))?;

        for row in rows {
            let ts = row?;
            let bucket = ((ts - window_start) / day_secs) as usize;
            if bucket < days {
                counts[bucket] += 1;
            }
        }

        Ok(counts)
    }

    /// Unified session search: FTS5 text search with LIKE fallback, plus metadata filters.
    pub fn search_sessions(
        &self,
        text: &str,
        project: Option<&str>,
        errors_only: bool,
        tool: Option<&str>,
    ) -> Result<Vec<SessionFile>> {
        let text = text.trim();

        if !text.is_empty() {
            // Try FTS5 first; on syntax errors (special chars like =, (, ), ", *)
            // silently fall through to LIKE instead of propagating the error.
            let fts_results = self
                .search_sessions_impl(text, project, errors_only, tool, true)
                .unwrap_or_default();
            if !fts_results.is_empty() {
                return Ok(fts_results);
            }
            // Fall back to LIKE (handles short/partial queries the FTS tokenizer skips,
            // and any input that caused an FTS syntax error above)
            self.search_sessions_impl(text, project, errors_only, tool, false)
        } else {
            // No text — apply metadata filters only
            self.search_sessions_impl("", project, errors_only, tool, false)
        }
    }

    fn search_sessions_impl(
        &self,
        text: &str,
        project: Option<&str>,
        errors_only: bool,
        tool: Option<&str>,
        use_fts: bool,
    ) -> Result<Vec<SessionFile>> {
        let mut joins = String::new();
        let mut conditions: Vec<String> =
            vec!["(s.first_message IS NOT NULL OR s.model IS NOT NULL)".to_string()];
        let mut params_storage: Vec<String> = Vec::new();

        if use_fts && !text.is_empty() {
            joins.push_str(" JOIN sessions_fts fts ON s.session_id = fts.session_id");
            conditions.push("sessions_fts MATCH ?".to_string());
            params_storage.push(text.to_string());
        } else if !text.is_empty() {
            let like_val = format!("%{}%", text);
            conditions.push("(s.first_message LIKE ? OR s.project_name LIKE ?)".to_string());
            params_storage.push(like_val.clone());
            params_storage.push(like_val);
        }

        if let Some(t) = tool {
            joins.push_str(" JOIN tool_usage tu ON tu.session_id = s.session_id");
            conditions.push("LOWER(tu.tool_name) = LOWER(?)".to_string());
            params_storage.push(t.to_string());
        }

        if let Some(p) = project {
            conditions.push("s.project_name = ?".to_string());
            params_storage.push(p.to_string());
        }

        if errors_only {
            conditions.push("s.error_count > 0".to_string());
        }

        let sql = format!(
            "SELECT DISTINCT {} FROM sessions s{} WHERE {} ORDER BY s.modified_at DESC",
            SESSION_COLS_PREFIXED,
            joins,
            conditions.join(" AND "),
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_storage
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let sessions = stmt
            .query_map(params_refs.as_slice(), session_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

}

/// Standard column list for all session SELECT queries.
const SESSION_COLS: &str = "session_id, project_name, file_path, file_size, created_at, \
                            modified_at, has_subagents, model, error_count, first_message, \
                            source_dir, subagent_models";

/// Same columns with `s.` table prefix (for JOINed queries).
const SESSION_COLS_PREFIXED: &str =
    "s.session_id, s.project_name, s.file_path, s.file_size, s.created_at, \
     s.modified_at, s.has_subagents, s.model, s.error_count, s.first_message, \
     s.source_dir, s.subagent_models";

/// Build a SessionFile from a row matching SESSION_COLS.
fn session_from_row(row: &rusqlite::Row) -> rusqlite::Result<SessionFile> {
    Ok(SessionFile {
        session_id: row.get(0)?,
        project_name: row.get(1)?,
        path: PathBuf::from(row.get::<_, String>(2)?),
        file_size: row.get::<_, i64>(3)? as u64,
        created_at: row.get::<_, i64>(4).unwrap_or(0),
        modified_at: row.get(5)?,
        has_subagents: row.get::<_, i64>(6)? != 0,
        model: row.get(7).ok().flatten(),
        error_count: row.get::<_, i64>(8).unwrap_or(0) as usize,
        first_message: row.get(9).ok().flatten(),
        source_dir: row.get::<_, String>(10).unwrap_or_default(),
        subagent_models: row.get(11).ok().flatten(),
    })
}

/// Extract a file path from a tool's JSON input, if the tool operates on files.
/// Returns None for tools that don't target specific files (Bash, Task, etc.).
fn file_path_from_input(tool_name: &str, input: &serde_json::Value) -> Option<String> {
    match tool_name {
        "Read" | "Write" | "Edit" | "NotebookEdit" => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        _ => None,
    }
}

/// Statistics for a project
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub project_name: String,
    pub session_count: usize,
    pub total_size: u64,
    pub last_activity: Option<i64>,
}

/// Global analytics across all sessions
#[derive(Debug, Clone)]
pub struct GlobalAnalytics {
    pub total_sessions: usize,
    pub sessions_this_week: usize,
    pub sessions_today: usize,
    pub total_size: u64,
    pub total_projects: usize,
    pub subagent_count: usize,
    pub avg_session_size: u64,
    pub most_active_project: Option<String>,
    pub top_tools: Vec<(String, usize)>,
    pub total_errors: usize,
}

/// Project-specific analytics
#[derive(Debug, Clone)]
pub struct ProjectAnalytics {
    pub project_name: String,
    pub total_sessions: usize,
    pub sessions_this_week: usize,
    pub sessions_today: usize,
    pub total_size: u64,
    pub subagent_count: usize,
    pub avg_session_size: u64,
    pub top_tools: Vec<(String, usize)>,
    pub last_activity: Option<i64>,
    pub total_errors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_creation() {
        let _temp_dir = TempDir::new().unwrap();
        let index = SessionIndex::new();
        assert!(index.is_ok());
    }

    #[test]
    fn test_index_session() {
        use crate::parser::models::{Message, MessageContent};
        use crate::parser::ExecutionNode;
        use std::collections::HashMap;
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("test-session.jsonl");

        // Write a minimal user message so the session passes the content guard
        let user_node = ExecutionNode {
            uuid: Some("u1".to_string()),
            parent_uuid: None,
            timestamp: Some(1_000),
            node_type: "user".to_string(),
            message: Some(Message {
                id: None,
                role: Some("user".to_string()),
                model: None,
                content: Some(MessageContent::Text("hello world".to_string())),
                usage: None,
                extra: HashMap::new(),
            }),
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        };
        let mut content = serde_json::to_string(&user_node).unwrap();
        content.push('\n');
        fs::write(&session_path, content).unwrap();

        let mut index = SessionIndex::new_in_memory().unwrap();

        let session = SessionFile {
            session_id: "test-session-123".to_string(),
            project_name: "test-project".to_string(),
            path: session_path,
            file_size: 1024,
            created_at: 1234567890,
            modified_at: 1234567890,
            has_subagents: false,
            model: None,
            error_count: 0,
            first_message: None,
            source_dir: String::new(),
            subagent_models: None,
        };

        let result = index.index_session(&session);
        assert!(result.is_ok());

        let count = index.count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_tool_usage_table() {
        use crate::parser::models::{Message, MessageContent, ToolUse};
        use crate::parser::ExecutionNode;
        use std::collections::HashMap;
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("test-session.jsonl");

        // Create a test session with tool usage
        let nodes = vec![
            // User message — needed to pass the empty-session guard in index_session
            ExecutionNode {
                uuid: Some("user1".to_string()),
                parent_uuid: None,
                timestamp: Some(500),
                node_type: "user".to_string(),
                message: Some(Message {
                    id: None,
                    role: Some("user".to_string()),
                    model: None,
                    content: Some(MessageContent::Text("please help me".to_string())),
                    usage: None,
                    extra: HashMap::new(),
                }),
                tool_use: None,
                tool_result: None,
                tool_use_result: None,
                thinking: None,
                progress: None,
                token_usage: None,
                extra: None,
            },
            ExecutionNode {
                uuid: Some("node1".to_string()),
                parent_uuid: None,
                timestamp: Some(1000),
                node_type: "tool_use".to_string(),
                message: None,
                tool_use: Some(ToolUse {
                    name: "Read".to_string(),
                    input: serde_json::json!({"file": "test.rs"}),
                    id: Some("tool1".to_string()),
                }),
                tool_result: None,
                tool_use_result: None,
                thinking: None,
                progress: None,
                token_usage: None,
                extra: None,
            },
            ExecutionNode {
                uuid: Some("node2".to_string()),
                parent_uuid: None,
                timestamp: Some(2000),
                node_type: "tool_use".to_string(),
                message: None,
                tool_use: Some(ToolUse {
                    name: "Edit".to_string(),
                    input: serde_json::json!({"file": "test.rs"}),
                    id: Some("tool2".to_string()),
                }),
                tool_result: None,
                tool_use_result: None,
                thinking: None,
                progress: None,
                token_usage: None,
                extra: None,
            },
            ExecutionNode {
                uuid: Some("node3".to_string()),
                parent_uuid: None,
                timestamp: Some(3000),
                node_type: "tool_use".to_string(),
                message: None,
                tool_use: Some(ToolUse {
                    name: "Read".to_string(),
                    input: serde_json::json!({"file": "main.rs"}),
                    id: Some("tool3".to_string()),
                }),
                tool_result: None,
                tool_use_result: None,
                thinking: None,
                progress: None,
                token_usage: None,
                extra: None,
            },
        ];

        // Write to JSONL file
        let mut file_content = String::new();
        for node in &nodes {
            file_content.push_str(&serde_json::to_string(node).unwrap());
            file_content.push('\n');
        }
        fs::write(&session_path, file_content).unwrap();

        // Create index and index the session
        let mut index = SessionIndex::new_in_memory().unwrap();
        let session = SessionFile {
            session_id: "test-session".to_string(),
            project_name: "test-project".to_string(),
            path: session_path,
            file_size: 1024,
            created_at: 1234567890,
            modified_at: 1234567890,
            has_subagents: false,
            model: None,
            error_count: 0,
            first_message: None,
            source_dir: String::new(),
            subagent_models: None,
        };

        index.index_session(&session).unwrap();

        // Query tool usage
        let top_tools = index.get_top_tools(100).unwrap();

        // Should have 2 unique tools: Read (2 times) and Edit (1 time)
        assert_eq!(top_tools.len(), 2);
        assert_eq!(top_tools[0].0, "Read");
        assert_eq!(top_tools[0].1, 2);
        assert_eq!(top_tools[1].0, "Edit");
        assert_eq!(top_tools[1].1, 1);
    }

    #[test]
    fn test_get_top_tools_empty() {
        let index = SessionIndex::new_in_memory().unwrap();
        let top_tools = index.get_top_tools(100).unwrap();
        assert_eq!(top_tools.len(), 0);
    }
}
