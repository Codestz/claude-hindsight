//! Session indexing with SQLite
//!
//! Maintains a fast SQLite index of all discovered sessions for quick lookups.

use crate::error::{HindsightError, Result};
use crate::storage::SessionFile;
use rusqlite::{Connection, params};
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
        let config_dir = dirs::config_dir()
            .ok_or_else(|| HindsightError::Config("Could not determine config directory".to_string()))?;

        let hindsight_dir = config_dir.join("hindsight");
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
        let version: i64 = self.conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        // Create tables if they don't exist
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                project_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                has_subagents INTEGER NOT NULL,
                indexed_at INTEGER NOT NULL
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
            "#,
        )?;

        // Set schema version to 1 if this is a new database
        if version == 0 {
            self.conn.execute("PRAGMA user_version = 1", [])?;
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

        // Start a transaction for atomic updates
        let tx = self.conn.transaction()?;

        // Insert/update session metadata
        tx.execute(
            r#"
            INSERT OR REPLACE INTO sessions
            (session_id, project_name, file_path, file_size, modified_at, has_subagents, indexed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                session.session_id,
                session.project_name,
                session.path.to_string_lossy(),
                session.file_size as i64,
                session.modified_at,
                if session.has_subagents { 1 } else { 0 },
                indexed_at,
            ],
        )?;

        // Parse session to extract tool counts
        if let Ok(parsed_session) = crate::parser::parse_session(&session.path) {
            let mut tool_counts: HashMap<String, usize> = HashMap::new();

            // Count tools from all nodes
            for node in &parsed_session.nodes {
                // Check top-level tool_use
                if let Some(ref tool_use) = node.tool_use {
                    *tool_counts.entry(tool_use.name.clone()).or_insert(0) += 1;
                }

                // Check message.content[] for tool_use blocks
                if let Some(ref message) = node.message {
                    if let Some(ref content) = message.content {
                        if let Some(content_array) = content.as_array() {
                            for content_item in content_array {
                                if let Some(content_type) = content_item.get("type").and_then(|v| v.as_str()) {
                                    if content_type == "tool_use" {
                                        if let Some(tool_name) = content_item.get("name").and_then(|v| v.as_str()) {
                                            *tool_counts.entry(tool_name.to_string()).or_insert(0) += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Delete old tool_usage entries for this session
            tx.execute(
                "DELETE FROM tool_usage WHERE session_id = ?1",
                params![session.session_id],
            )?;

            // Insert new tool_usage entries
            let mut stmt = tx.prepare(
                "INSERT INTO tool_usage (session_id, tool_name, usage_count) VALUES (?1, ?2, ?3)"
            )?;

            for (tool_name, count) in tool_counts {
                stmt.execute(params![session.session_id, tool_name, count as i64])?;
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
            "SELECT session_id, project_name, file_path, file_size, modified_at, has_subagents
             FROM sessions
             ORDER BY modified_at DESC"
        )?;

        let sessions = stmt.query_map([], |row| {
            Ok(SessionFile {
                session_id: row.get(0)?,
                project_name: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                file_size: row.get::<_, i64>(3)? as u64,
                modified_at: row.get(4)?,
                has_subagents: row.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Find sessions by project name
    pub fn find_by_project(&self, project: &str) -> Result<Vec<SessionFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, project_name, file_path, file_size, modified_at, has_subagents
             FROM sessions
             WHERE project_name = ?1
             ORDER BY modified_at DESC"
        )?;

        let sessions = stmt.query_map([project], |row| {
            Ok(SessionFile {
                session_id: row.get(0)?,
                project_name: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                file_size: row.get::<_, i64>(3)? as u64,
                modified_at: row.get(4)?,
                has_subagents: row.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Find a session by ID (exact match or prefix)
    pub fn find_by_id(&self, session_id: &str) -> Result<Option<SessionFile>> {
        // Try exact match first
        let mut stmt = self.conn.prepare(
            "SELECT session_id, project_name, file_path, file_size, modified_at, has_subagents
             FROM sessions
             WHERE session_id = ?1"
        )?;

        let result = stmt.query_row([session_id], |row| {
            Ok(SessionFile {
                session_id: row.get(0)?,
                project_name: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                file_size: row.get::<_, i64>(3)? as u64,
                modified_at: row.get(4)?,
                has_subagents: row.get::<_, i64>(5)? != 0,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Try prefix match
                let mut stmt = self.conn.prepare(
                    "SELECT session_id, project_name, file_path, file_size, modified_at, has_subagents
                     FROM sessions
                     WHERE session_id LIKE ?1 || '%'
                     ORDER BY modified_at DESC
                     LIMIT 1"
                )?;

                let result = stmt.query_row([session_id], |row| {
                    Ok(SessionFile {
                        session_id: row.get(0)?,
                        project_name: row.get(1)?,
                        path: PathBuf::from(row.get::<_, String>(2)?),
                        file_size: row.get::<_, i64>(3)? as u64,
                        modified_at: row.get(4)?,
                        has_subagents: row.get::<_, i64>(5)? != 0,
                    })
                });

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
    #[allow(dead_code)]
    pub fn get_latest(&self) -> Result<Option<SessionFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, project_name, file_path, file_size, modified_at, has_subagents
             FROM sessions
             ORDER BY modified_at DESC
             LIMIT 1"
        )?;

        let result = stmt.query_row([], |row| {
            Ok(SessionFile {
                session_id: row.get(0)?,
                project_name: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                file_size: row.get::<_, i64>(3)? as u64,
                modified_at: row.get(4)?,
                has_subagents: row.get::<_, i64>(5)? != 0,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Remove sessions that no longer exist on disk
    #[allow(dead_code)]
    pub fn prune_missing(&mut self) -> Result<usize> {
        let sessions = self.list_sessions()?;
        let mut removed = 0;

        for session in sessions {
            if !session.path.exists() {
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
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Get all unique project names
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT project_name FROM sessions ORDER BY project_name"
        )?;

        let projects = stmt.query_map([], |row| row.get(0))?
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
             WHERE project_name = ?1"
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

        // Total sessions and size
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(file_size) FROM sessions"
        )?;

        let (total_sessions, total_size) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
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
        let most_active_project = self.conn.query_row(
            "SELECT project_name FROM sessions ORDER BY modified_at DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        ).ok();

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
            LIMIT 5
            "#
        )?;

        let top_tools = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as usize,
            ))
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

        // Total sessions and size for this project
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(file_size) FROM sessions WHERE project_name = ?1"
        )?;

        let (total_sessions, total_size) = stmt.query_row([project], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
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
        })
    }

    /// Get top tools for a specific project
    fn get_top_tools_for_project(&self, project: &str, _session_limit: usize) -> Result<Vec<(String, usize)>> {
        // Query aggregates tool usage for this project (no file parsing!)
        let mut stmt = self.conn.prepare(
            r#"
            SELECT t.tool_name, SUM(t.usage_count) as total_count
            FROM tool_usage t
            JOIN sessions s ON t.session_id = s.session_id
            WHERE s.project_name = ?1
            GROUP BY t.tool_name
            ORDER BY total_count DESC
            LIMIT 5
            "#
        )?;

        let top_tools = stmt.query_map([project], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as usize,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(top_tools)
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
}

/// Project-specific analytics
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
        let mut index = SessionIndex::new_in_memory().unwrap();

        let session = SessionFile {
            session_id: "test-session-123".to_string(),
            project_name: "test-project".to_string(),
            path: PathBuf::from("/tmp/test.jsonl"),
            file_size: 1024,
            modified_at: 1234567890,
            has_subagents: false,
        };

        let result = index.index_session(&session);
        assert!(result.is_ok());

        let count = index.count().unwrap();
        assert_eq!(count, 1);
    }
}
