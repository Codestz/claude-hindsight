//! Session indexing with SQLite
//!
//! Maintains a fast SQLite index of all discovered sessions for quick lookups.

use crate::error::{HindsightError, Result};
use crate::storage::SessionFile;
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// The expected schema version for this release.
/// When a user's DB has a different version, we delete and rebuild automatically.
const SCHEMA_VERSION: i64 = 3;

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

        let (index, needs_reindex) = Self::open(&db_path)?;

        // After a schema upgrade, do a full reindex so every command starts
        // with a fully populated database — no manual steps required.
        if needs_reindex {
            Self::reindex_after_upgrade(index)
        } else {
            Ok(index)
        }
    }

    /// Open (or create) a database at the given path, handling schema migration.
    ///
    /// Returns the index and whether a schema upgrade forced a DB rebuild
    /// (callers should reindex when `true`).
    fn open(db_path: &std::path::Path) -> Result<(Self, bool)> {
        // Detect schema version mismatch and rebuild if needed.
        // This handles upgrades from v1.x where the schema layout is incompatible.
        let needs_reindex = if db_path.exists() {
            let conn = Connection::open(db_path)?;
            let version: i64 =
                conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            drop(conn);

            if version != 0 && version != SCHEMA_VERSION {
                eprintln!(
                    "  Upgrading database schema (v{} -> v{}). Rebuilding index...",
                    version, SCHEMA_VERSION
                );
                std::fs::remove_file(db_path)?;
                true
            } else {
                false
            }
        } else {
            false
        };

        let conn = Connection::open(db_path)?;

        let mut index = SessionIndex { conn };
        index.initialize_schema()?;

        Ok((index, needs_reindex))
    }

    /// Discover all sessions and index them after a schema upgrade.
    fn reindex_after_upgrade(mut index: Self) -> Result<Self> {
        match crate::storage::discover_sessions() {
            Ok(sessions) => {
                let mut indexed = 0usize;
                for session in &sessions {
                    if index.index_session(session).is_ok() {
                        indexed += 1;
                    }
                }
                eprintln!("  Reindexed {} session(s). Ready!", indexed);
            }
            Err(crate::error::HindsightError::NoSessionsFound) => {
                eprintln!("  No sessions found. Run 'hindsight init' after your first Claude Code session.");
            }
            Err(e) => {
                eprintln!("  Warning: could not reindex sessions: {}", e);
            }
        }
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
            // Fresh database: create all tables at latest schema version
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
                    subagent_models TEXT,
                    input_tokens INTEGER NOT NULL DEFAULT 0,
                    output_tokens INTEGER NOT NULL DEFAULT 0,
                    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
                    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
                    cost_usd REAL NOT NULL DEFAULT 0.0
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

                CREATE TABLE IF NOT EXISTS otel_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at INTEGER NOT NULL,
                    session_id TEXT,
                    event_name TEXT NOT NULL,
                    model TEXT,
                    cost_usd REAL,
                    input_tokens INTEGER,
                    output_tokens INTEGER,
                    cache_read_tokens INTEGER,
                    cache_creation_tokens INTEGER,
                    duration_ms INTEGER,
                    attributes TEXT
                );

                CREATE TABLE IF NOT EXISTS hook_tool_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    hook_event TEXT NOT NULL,
                    tool_name TEXT,
                    tool_input TEXT,
                    tool_result TEXT,
                    error_message TEXT,
                    is_interrupt INTEGER,
                    tool_use_id TEXT,
                    cwd TEXT,
                    source TEXT NOT NULL DEFAULT 'hook'
                );

                CREATE TABLE IF NOT EXISTS hook_subagent_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    hook_event TEXT NOT NULL,
                    agent_type TEXT,
                    agent_name TEXT,
                    cwd TEXT
                );

                CREATE TABLE IF NOT EXISTS hook_compaction_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    compaction_trigger TEXT
                );

                CREATE TABLE IF NOT EXISTS hook_permission_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    tool_name TEXT,
                    tool_input TEXT,
                    cwd TEXT
                );

                CREATE TABLE IF NOT EXISTS hook_lifecycle_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    event_name TEXT NOT NULL,
                    attributes TEXT
                );

                CREATE TABLE IF NOT EXISTS otel_metrics (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at     INTEGER NOT NULL,
                    session_id      TEXT,
                    metric_name     TEXT NOT NULL,
                    token_type      TEXT,
                    model           TEXT,
                    value_int       INTEGER,
                    value_double    REAL,
                    time_unix_nano  TEXT,
                    service_name    TEXT,
                    service_version TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_session ON otel_metrics(session_id);
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_name    ON otel_metrics(metric_name);
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_time    ON otel_metrics(received_at DESC);

                CREATE TABLE IF NOT EXISTS otel_logs (
                    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at           INTEGER NOT NULL,
                    session_id            TEXT,
                    event_name            TEXT,
                    model                 TEXT,
                    cost_usd              REAL,
                    input_tokens          INTEGER,
                    output_tokens         INTEGER,
                    cache_read_tokens     INTEGER,
                    cache_creation_tokens INTEGER,
                    duration_ms           INTEGER,
                    tool_name             TEXT,
                    success               INTEGER,
                    error_message         TEXT,
                    status_code           INTEGER,
                    severity              TEXT,
                    body                  TEXT,
                    attributes            TEXT,
                    time_unix_nano        TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_otel_logs_session ON otel_logs(session_id);
                CREATE INDEX IF NOT EXISTS idx_otel_logs_event   ON otel_logs(event_name);
                CREATE INDEX IF NOT EXISTS idx_otel_logs_time    ON otel_logs(received_at DESC);
                "#,
            )?;
            self.conn
                .execute(&format!("PRAGMA user_version = {}", SCHEMA_VERSION), [])?;
        } else if version < SCHEMA_VERSION {
            // Single migration from v1 → v2: add all columns and tables
            // Use `let _ =` for ALTER TABLE to handle columns that may already exist
            for stmt in &[
                "ALTER TABLE sessions ADD COLUMN model TEXT",
                "ALTER TABLE sessions ADD COLUMN error_count INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN first_message TEXT",
                "ALTER TABLE sessions ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN source_dir TEXT NOT NULL DEFAULT ''",
                "ALTER TABLE sessions ADD COLUMN subagent_models TEXT",
                "ALTER TABLE sessions ADD COLUMN input_tokens INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN output_tokens INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN cache_read_tokens INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN cache_creation_tokens INTEGER NOT NULL DEFAULT 0",
                "ALTER TABLE sessions ADD COLUMN cost_usd REAL NOT NULL DEFAULT 0.0",
            ] {
                let _ = self.conn.execute(stmt, []);
            }

            let _ = self.conn.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(session_id UNINDEXED, searchable_text, tokenize='porter ascii');"
            );

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

                CREATE TABLE IF NOT EXISTS otel_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at INTEGER NOT NULL,
                    session_id TEXT,
                    event_name TEXT NOT NULL,
                    model TEXT,
                    cost_usd REAL,
                    input_tokens INTEGER,
                    output_tokens INTEGER,
                    cache_read_tokens INTEGER,
                    cache_creation_tokens INTEGER,
                    duration_ms INTEGER,
                    attributes TEXT
                );

                CREATE TABLE IF NOT EXISTS hook_tool_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    hook_event TEXT NOT NULL,
                    tool_name TEXT,
                    tool_input TEXT,
                    tool_result TEXT,
                    error_message TEXT,
                    is_interrupt INTEGER,
                    tool_use_id TEXT,
                    cwd TEXT,
                    source TEXT NOT NULL DEFAULT 'hook'
                );
                CREATE TABLE IF NOT EXISTS hook_subagent_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    hook_event TEXT NOT NULL,
                    agent_type TEXT,
                    agent_name TEXT,
                    cwd TEXT
                );
                CREATE TABLE IF NOT EXISTS hook_compaction_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    compaction_trigger TEXT
                );
                CREATE TABLE IF NOT EXISTS hook_permission_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    tool_name TEXT,
                    tool_input TEXT,
                    cwd TEXT
                );
                CREATE TABLE IF NOT EXISTS hook_lifecycle_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    occurred_at INTEGER NOT NULL,
                    event_name TEXT NOT NULL,
                    attributes TEXT
                );

                CREATE TABLE IF NOT EXISTS otel_metrics (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at     INTEGER NOT NULL,
                    session_id      TEXT,
                    metric_name     TEXT NOT NULL,
                    token_type      TEXT,
                    model           TEXT,
                    value_int       INTEGER,
                    value_double    REAL,
                    time_unix_nano  TEXT,
                    service_name    TEXT,
                    service_version TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_session ON otel_metrics(session_id);
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_name    ON otel_metrics(metric_name);
                CREATE INDEX IF NOT EXISTS idx_otel_metrics_time    ON otel_metrics(received_at DESC);

                CREATE TABLE IF NOT EXISTS otel_logs (
                    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
                    received_at           INTEGER NOT NULL,
                    session_id            TEXT,
                    event_name            TEXT,
                    model                 TEXT,
                    cost_usd              REAL,
                    input_tokens          INTEGER,
                    output_tokens         INTEGER,
                    cache_read_tokens     INTEGER,
                    cache_creation_tokens INTEGER,
                    duration_ms           INTEGER,
                    tool_name             TEXT,
                    success               INTEGER,
                    error_message         TEXT,
                    status_code           INTEGER,
                    severity              TEXT,
                    body                  TEXT,
                    attributes            TEXT,
                    time_unix_nano        TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_otel_logs_session ON otel_logs(session_id);
                CREATE INDEX IF NOT EXISTS idx_otel_logs_event   ON otel_logs(event_name);
                CREATE INDEX IF NOT EXISTS idx_otel_logs_time    ON otel_logs(received_at DESC);
                "#,
            )?;
            self.conn
                .execute(&format!("PRAGMA user_version = {}", SCHEMA_VERSION), [])?;
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
        // Each JSONL tool event extracted during parsing
        struct JsonlToolEvent {
            occurred_at: i64,
            hook_event: String, // "ToolUse" or "ToolResult"
            tool_name: Option<String>,
            tool_input: Option<String>,
            tool_result: Option<String>,
            error_message: Option<String>,
            tool_use_id: Option<String>,
        }

        let (model, error_count, first_message, tool_counts, file_counts, created_at,
             input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost_usd,
             jsonl_tool_events) =
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
                    .filter(|n| n.node_type == crate::parser::models::NodeType::User)
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

                // Aggregate token usage across all assistant nodes
                let mut agg_input: i64 = 0;
                let mut agg_output: i64 = 0;
                let mut agg_cache_read: i64 = 0;
                let mut agg_cache_creation: i64 = 0;
                for node in &parsed.nodes {
                    if let Some(tu) = node.effective_token_usage() {
                        agg_input += tu.input_tokens.unwrap_or(0);
                        agg_output += tu.output_tokens.unwrap_or(0);
                        agg_cache_read += tu.cache_read_input_tokens.unwrap_or(0);
                        agg_cache_creation += tu.cache_creation_input_tokens.unwrap_or(0);
                    }
                }
                // Model-aware cost: each node priced by its actual model,
                // falling back to the session's primary model.
                let cost = crate::pricing::calculate_session_cost(
                    &parsed.nodes,
                    parsed.model.as_deref(),
                );

                // Extract individual tool events from JSONL nodes for Activity page.
                // Build a map of tool_use_id → tool_result for pairing.
                type ToolResult = (Option<String>, Option<String>, Option<bool>);
                let mut result_map: HashMap<String, ToolResult> = HashMap::new();
                for node in &parsed.nodes {
                    // Top-level tool_result
                    if let Some(ref tr) = node.tool_result {
                        if let Some(ref tuid) = tr.tool_use_id {
                            let content = tr.content.clone();
                            let err = tr.error.clone();
                            let is_err = tr.is_error;
                            result_map.insert(tuid.clone(), (content, err, is_err));
                        }
                    }
                    // ContentBlock::ToolResult
                    for block in node.message.as_ref().map(|m| m.content_blocks()).unwrap_or(&[]) {
                        if let crate::parser::models::ContentBlock::ToolResult { tool_use_id, content, is_error } = block {
                            let content_str = content.as_ref().map(|v| {
                                if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() }
                            });
                            result_map.insert(tool_use_id.clone(), (content_str, None, *is_error));
                        }
                    }
                }

                let mut jte: Vec<JsonlToolEvent> = Vec::new();
                for node in &parsed.nodes {
                    let ts = node.timestamp.map(|ms| ms / 1000).unwrap_or(created_at);

                    // Top-level tool_use
                    if let Some(ref tu) = node.tool_use {
                        let input_str = serde_json::to_string(&tu.input).ok();
                        let tuid = tu.id.clone();
                        let (result, error, _is_err) = tuid.as_ref()
                            .and_then(|id| result_map.get(id))
                            .cloned()
                            .unwrap_or((None, None, None));
                        jte.push(JsonlToolEvent {
                            occurred_at: ts,
                            hook_event: "PostToolUse".to_string(),
                            tool_name: Some(tu.name.clone()),
                            tool_input: input_str,
                            tool_result: result,
                            error_message: error,
                            tool_use_id: tuid,
                        });
                    }

                    // ContentBlock::ToolUse
                    for block in node.message.as_ref().map(|m| m.content_blocks()).unwrap_or(&[]) {
                        if let crate::parser::models::ContentBlock::ToolUse { id, name, input } = block {
                            let input_str = serde_json::to_string(input).ok();
                            let (result, error, _is_err) = result_map.get(id.as_str())
                                .cloned()
                                .unwrap_or((None, None, None));
                            jte.push(JsonlToolEvent {
                                occurred_at: ts,
                                hook_event: "PostToolUse".to_string(),
                                tool_name: Some(name.clone()),
                                tool_input: input_str,
                                tool_result: result,
                                error_message: error,
                                tool_use_id: Some(id.clone()),
                            });
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
                    agg_input,
                    agg_output,
                    agg_cache_read,
                    agg_cache_creation,
                    cost,
                    jte,
                )
            } else {
                (
                    None::<String>,
                    0i64,
                    None::<String>,
                    None,
                    None,
                    session.modified_at,
                    0i64,
                    0i64,
                    0i64,
                    0i64,
                    0.0f64,
                    Vec::new(),
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
             model, error_count, first_message, source_dir, subagent_models,
             input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost_usd)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
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
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_creation_tokens,
                cost_usd,
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

        // Backfill tool events from JSONL — delete stale jsonl rows, then re-insert.
        // Hook-sourced rows are never touched.
        if !jsonl_tool_events.is_empty() {
            tx.execute(
                "DELETE FROM hook_tool_events WHERE session_id = ?1 AND source = 'jsonl'",
                params![session.session_id],
            )?;

            // Collect existing hook tool_use_ids so we skip duplicates
            let mut existing_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
            {
                let mut id_stmt = tx.prepare(
                    "SELECT tool_use_id FROM hook_tool_events WHERE session_id = ?1 AND source = 'hook' AND tool_use_id IS NOT NULL",
                )?;
                let rows = id_stmt.query_map(params![session.session_id], |row| row.get::<_, String>(0))?;
                for id in rows.flatten() {
                    existing_ids.insert(id);
                }
            }

            let mut ins_stmt = tx.prepare(
                "INSERT INTO hook_tool_events \
                 (session_id, occurred_at, hook_event, tool_name, tool_input, tool_result, \
                  error_message, is_interrupt, tool_use_id, cwd, source) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,NULL,?8,NULL,'jsonl')",
            )?;

            for evt in &jsonl_tool_events {
                // Skip if hook already captured this tool_use_id
                if let Some(ref tuid) = evt.tool_use_id {
                    if existing_ids.contains(tuid) {
                        continue;
                    }
                }
                ins_stmt.execute(params![
                    session.session_id,
                    evt.occurred_at,
                    evt.hook_event,
                    evt.tool_name,
                    evt.tool_input,
                    evt.tool_result,
                    evt.error_message,
                    evt.tool_use_id,
                ])?;
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

    /// Sync only new sessions (not already in the DB) — fast startup auto-index.
    ///
    /// Discovers all sessions on disk, compares against indexed session IDs, and
    /// indexes only the ones not already present. Does NOT re-parse existing sessions.
    /// Returns the number of newly indexed sessions.
    pub fn sync_new_only(&mut self) -> Result<usize> {
        // Collect known IDs into an owned set so we don't hold a borrow on self.conn
        let known_ids: std::collections::HashSet<String> = {
            let mut stmt = self.conn.prepare("SELECT session_id FROM sessions")?;
            let ids: std::result::Result<Vec<String>, _> =
                stmt.query_map([], |row| row.get(0))?.collect();
            ids?.into_iter().collect()
        };

        // Discover sessions on disk; tolerate "none found" gracefully
        let all_sessions = match crate::storage::discover_sessions() {
            Ok(sessions) => sessions,
            Err(crate::error::HindsightError::NoSessionsFound) => return Ok(0),
            Err(e) => return Err(e),
        };

        let new_sessions: Vec<_> = all_sessions
            .into_iter()
            .filter(|s| !known_ids.contains(&s.session_id))
            .collect();

        let count = new_sessions.len();
        for session in &new_sessions {
            // Swallow per-session errors; a bad file shouldn't block the command.
            let _ = self.index_session(session);
        }

        Ok(count)
    }

    // ── OTLP insert methods ───────────────────────────────────────────────────

    /// Insert a batch of parsed metric data points into `otel_metrics`.
    pub fn insert_otel_metrics(&self, records: &[crate::otel::OtelMetricRecord]) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO otel_metrics \
             (received_at,session_id,metric_name,token_type,model,value_int,value_double,\
              time_unix_nano,service_name,service_version) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        )?;
        for r in records {
            stmt.execute(params![
                r.received_at,
                r.session_id,
                r.metric_name,
                r.token_type,
                r.model,
                r.value_int,
                r.value_double,
                r.time_unix_nano,
                r.service_name,
                r.service_version,
            ])?;
        }
        Ok(())
    }

    /// Insert a batch of parsed log records into `otel_logs`.
    pub fn insert_otel_logs(&self, records: &[crate::otel::OtelLogRecord]) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO otel_logs \
             (received_at,session_id,event_name,model,cost_usd,input_tokens,output_tokens,\
              cache_read_tokens,cache_creation_tokens,duration_ms,tool_name,success,\
              error_message,status_code,severity,body,attributes,time_unix_nano) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        )?;
        for r in records {
            stmt.execute(params![
                r.received_at,
                r.session_id,
                r.event_name,
                r.model,
                r.cost_usd,
                r.input_tokens,
                r.output_tokens,
                r.cache_read_tokens,
                r.cache_creation_tokens,
                r.duration_ms,
                r.tool_name,
                r.success.map(|b| if b { 1i64 } else { 0i64 }),
                r.error_message,
                r.status_code,
                r.severity,
                r.body,
                r.attributes,
                r.time_unix_nano,
            ])?;
        }
        Ok(())
    }

    // ── OTLP query methods ────────────────────────────────────────────────────

    pub fn get_otel_metrics(
        &self,
        session_id: &str,
        metric: Option<&str>,
    ) -> Result<Vec<crate::otel::OtelMetricRecord>> {
        let has_session = !session_id.is_empty();

        let sql = match (has_session, metric.is_some()) {
            (true, true) => "SELECT received_at,session_id,metric_name,token_type,model,value_int,value_double,\
             time_unix_nano,service_name,service_version \
             FROM otel_metrics WHERE session_id=?1 AND metric_name=?2 ORDER BY received_at".to_string(),
            (true, false) => "SELECT received_at,session_id,metric_name,token_type,model,value_int,value_double,\
             time_unix_nano,service_name,service_version \
             FROM otel_metrics WHERE session_id=?1 ORDER BY received_at".to_string(),
            (false, true) => "SELECT received_at,session_id,metric_name,token_type,model,value_int,value_double,\
             time_unix_nano,service_name,service_version \
             FROM otel_metrics WHERE metric_name=?1 ORDER BY received_at".to_string(),
            (false, false) => "SELECT received_at,session_id,metric_name,token_type,model,value_int,value_double,\
             time_unix_nano,service_name,service_version \
             FROM otel_metrics ORDER BY received_at".to_string(),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mapper = |row: &rusqlite::Row| {
            Ok(crate::otel::OtelMetricRecord {
                received_at: row.get(0)?,
                session_id: row.get(1)?,
                metric_name: row.get(2)?,
                token_type: row.get(3)?,
                model: row.get(4)?,
                value_int: row.get(5)?,
                value_double: row.get(6)?,
                time_unix_nano: row.get(7)?,
                service_name: row.get(8)?,
                service_version: row.get(9)?,
            })
        };

        let rows = match (has_session, metric) {
            (true, Some(m)) => stmt.query_map(params![session_id, m], mapper)?,
            (true, None) => stmt.query_map(params![session_id], mapper)?,
            (false, Some(m)) => stmt.query_map(params![m], mapper)?,
            (false, None) => stmt.query_map([], mapper)?,
        }
        .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_otel_logs(
        &self,
        session_id: &str,
        event: Option<&str>,
    ) -> Result<Vec<crate::otel::OtelLogRecord>> {
        let has_session = !session_id.is_empty();
        let select = "SELECT received_at,session_id,event_name,model,cost_usd,input_tokens,output_tokens,\
             cache_read_tokens,cache_creation_tokens,duration_ms,tool_name,success,\
             error_message,status_code,severity,body,attributes,time_unix_nano \
             FROM otel_logs";

        let sql = match (has_session, event.is_some()) {
            (true, true) => format!("{select} WHERE session_id=?1 AND event_name=?2 ORDER BY received_at"),
            (true, false) => format!("{select} WHERE session_id=?1 ORDER BY received_at"),
            (false, true) => format!("{select} WHERE event_name=?1 ORDER BY received_at"),
            (false, false) => format!("{select} ORDER BY received_at"),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mapper = |row: &rusqlite::Row| {
            Ok(crate::otel::OtelLogRecord {
                received_at: row.get(0)?,
                session_id: row.get(1)?,
                event_name: row.get(2)?,
                model: row.get(3)?,
                cost_usd: row.get(4)?,
                input_tokens: row.get(5)?,
                output_tokens: row.get(6)?,
                cache_read_tokens: row.get(7)?,
                cache_creation_tokens: row.get(8)?,
                duration_ms: row.get(9)?,
                tool_name: row.get(10)?,
                success: row.get::<_, Option<i64>>(11)?.map(|v| v != 0),
                error_message: row.get(12)?,
                status_code: row.get(13)?,
                severity: row.get(14)?,
                body: row.get(15)?,
                attributes: row.get(16)?,
                time_unix_nano: row.get(17)?,
            })
        };

        let rows = match (has_session, event) {
            (true, Some(e)) => stmt.query_map(params![session_id, e], mapper)?,
            (true, None) => stmt.query_map(params![session_id], mapper)?,
            (false, Some(e)) => stmt.query_map(params![e], mapper)?,
            (false, None) => stmt.query_map([], mapper)?,
        }
        .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Aggregated tokens + cost from `otel_logs` for a single session.
    pub fn get_otel_session_summary(&self, session_id: &str) -> Result<OtelSessionSummary> {
        let row = self.conn.query_row(
            "SELECT \
                COALESCE(SUM(input_tokens),0), \
                COALESCE(SUM(output_tokens),0), \
                COALESCE(SUM(cache_read_tokens),0), \
                COALESCE(SUM(cache_creation_tokens),0), \
                COALESCE(SUM(cost_usd),0.0), \
                COUNT(*) \
             FROM otel_logs \
             WHERE session_id=?1 AND event_name='api_request'",
            params![session_id],
            |row| {
                Ok(OtelSessionSummary {
                    session_id: session_id.to_owned(),
                    input_tokens: row.get(0)?,
                    output_tokens: row.get(1)?,
                    cache_read_tokens: row.get(2)?,
                    cache_creation_tokens: row.get(3)?,
                    cost_usd: row.get(4)?,
                    api_requests: row.get::<_, i64>(5)? as usize,
                })
            },
        )?;
        Ok(row)
    }

    /// Aggregate across all sessions from `otel_logs`.
    pub fn get_otel_global_summary(&self) -> Result<OtelGlobalSummary> {
        let row = self.conn.query_row(
            "SELECT \
                COUNT(DISTINCT session_id), \
                COALESCE(SUM(input_tokens),0), \
                COALESCE(SUM(output_tokens),0), \
                COALESCE(SUM(cache_read_tokens),0), \
                COALESCE(SUM(cache_creation_tokens),0), \
                COALESCE(SUM(cost_usd),0.0), \
                COUNT(*) \
             FROM otel_logs \
             WHERE event_name='api_request'",
            [],
            |row| {
                Ok(OtelGlobalSummary {
                    total_sessions: row.get::<_, i64>(0)? as usize,
                    input_tokens: row.get(1)?,
                    output_tokens: row.get(2)?,
                    cache_read_tokens: row.get(3)?,
                    cache_creation_tokens: row.get(4)?,
                    cost_usd: row.get(5)?,
                    api_requests: row.get::<_, i64>(6)? as usize,
                })
            },
        )?;
        Ok(row)
    }

    /// Get aggregated telemetry summary — prefers OTLP data, falls back to JSONL.
    pub fn get_telemetry_summary(&self) -> Result<crate::server::routes::telemetry::TelemetrySummary> {
        // Check if we have any OTLP log data for api_request events.
        let otel_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM otel_logs WHERE event_name='api_request'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        if otel_count > 0 {
            // Use OTLP data (more accurate — real-time from Claude Code telemetry).
            let row = self.conn.query_row(
                "SELECT \
                    COUNT(DISTINCT session_id), \
                    COALESCE(SUM(input_tokens),0), \
                    COALESCE(SUM(output_tokens),0), \
                    COALESCE(SUM(cache_read_tokens),0), \
                    COALESCE(SUM(cache_creation_tokens),0), \
                    COALESCE(SUM(cost_usd),0.0) \
                 FROM otel_logs \
                 WHERE event_name='api_request'",
                [],
                |row| {
                    Ok(crate::server::routes::telemetry::TelemetrySummary {
                        total_sessions: row.get::<_, i64>(0)? as usize,
                        input_tokens: row.get(1)?,
                        output_tokens: row.get(2)?,
                        cache_read_tokens: row.get(3)?,
                        cache_creation_tokens: row.get(4)?,
                        cost_usd: row.get(5)?,
                    })
                },
            )?;
            return Ok(row);
        }

        // Fall back to JSONL-derived columns in the sessions table.
        let row = self.conn.query_row(
            "SELECT COUNT(*), \
                    COALESCE(SUM(input_tokens),0), \
                    COALESCE(SUM(output_tokens),0), \
                    COALESCE(SUM(cache_read_tokens),0), \
                    COALESCE(SUM(cache_creation_tokens),0), \
                    COALESCE(SUM(cost_usd),0.0) \
             FROM sessions",
            [],
            |row| {
                Ok(crate::server::routes::telemetry::TelemetrySummary {
                    total_sessions: row.get::<_, i64>(0)? as usize,
                    input_tokens: row.get(1)?,
                    output_tokens: row.get(2)?,
                    cache_read_tokens: row.get(3)?,
                    cache_creation_tokens: row.get(4)?,
                    cost_usd: row.get(5)?,
                })
            },
        )?;
        Ok(row)
    }

    /// Get per-session telemetry — prefers OTLP data, falls back to JSONL.
    pub fn get_telemetry_per_session(&self) -> Result<Vec<crate::server::routes::telemetry::SessionTelemetry>> {
        // Check if we have any OTLP log data.
        let otel_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM otel_logs WHERE event_name='api_request'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        if otel_count > 0 {
            // Use OTLP data grouped by session; join sessions for project_name.
            let mut stmt = self.conn.prepare(
                "SELECT l.session_id, \
                        COALESCE(s.project_name, l.session_id), \
                        COALESCE(SUM(l.input_tokens),0), \
                        COALESCE(SUM(l.output_tokens),0), \
                        COALESCE(SUM(l.cache_read_tokens),0), \
                        COALESCE(SUM(l.cache_creation_tokens),0), \
                        COALESCE(SUM(l.cost_usd),0.0) \
                 FROM otel_logs l \
                 LEFT JOIN sessions s ON l.session_id = s.session_id \
                 WHERE l.event_name='api_request' \
                 GROUP BY l.session_id \
                 ORDER BY SUM(l.cost_usd) DESC",
            )?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(crate::server::routes::telemetry::SessionTelemetry {
                        session_id: row.get(0)?,
                        project_name: row.get(1)?,
                        input_tokens: row.get(2)?,
                        output_tokens: row.get(3)?,
                        cache_read_tokens: row.get(4)?,
                        cache_creation_tokens: row.get(5)?,
                        cost_usd: row.get(6)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            return Ok(rows);
        }

        // Fall back to JSONL-derived sessions table.
        let mut stmt = self.conn.prepare(
            "SELECT session_id, project_name, \
                    COALESCE(input_tokens,0), COALESCE(output_tokens,0), \
                    COALESCE(cache_read_tokens,0), COALESCE(cache_creation_tokens,0), \
                    COALESCE(cost_usd,0.0) \
             FROM sessions \
             ORDER BY cost_usd DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(crate::server::routes::telemetry::SessionTelemetry {
                    session_id: row.get(0)?,
                    project_name: row.get(1)?,
                    input_tokens: row.get(2)?,
                    output_tokens: row.get(3)?,
                    cache_read_tokens: row.get(4)?,
                    cache_creation_tokens: row.get(5)?,
                    cost_usd: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
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

    // ── Hook event insert methods ─────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_hook_tool_event(
        &self,
        session_id: &str,
        hook_event: &str,
        tool_name: Option<&str>,
        tool_input: Option<&str>,
        tool_result: Option<&str>,
        error_message: Option<&str>,
        is_interrupt: Option<bool>,
        tool_use_id: Option<&str>,
        cwd: Option<&str>,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO hook_tool_events \
             (session_id, occurred_at, hook_event, tool_name, tool_input, tool_result, \
              error_message, is_interrupt, tool_use_id, cwd) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                session_id,
                now,
                hook_event,
                tool_name,
                tool_input,
                tool_result,
                error_message,
                is_interrupt.map(|b| if b { 1i64 } else { 0i64 }),
                tool_use_id,
                cwd,
            ],
        )?;
        Ok(())
    }

    pub fn insert_hook_subagent_event(
        &self,
        session_id: &str,
        hook_event: &str,
        agent_type: Option<&str>,
        agent_name: Option<&str>,
        cwd: Option<&str>,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO hook_subagent_events \
             (session_id, occurred_at, hook_event, agent_type, agent_name, cwd) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![session_id, now, hook_event, agent_type, agent_name, cwd],
        )?;
        Ok(())
    }

    pub fn insert_hook_compaction_event(
        &self,
        session_id: &str,
        compaction_trigger: Option<&str>,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO hook_compaction_events (session_id, occurred_at, compaction_trigger) \
             VALUES (?1,?2,?3)",
            params![session_id, now, compaction_trigger],
        )?;
        Ok(())
    }

    pub fn insert_hook_permission_event(
        &self,
        session_id: &str,
        tool_name: Option<&str>,
        tool_input: Option<&str>,
        cwd: Option<&str>,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO hook_permission_events \
             (session_id, occurred_at, tool_name, tool_input, cwd) VALUES (?1,?2,?3,?4,?5)",
            params![session_id, now, tool_name, tool_input, cwd],
        )?;
        Ok(())
    }

    pub fn insert_hook_lifecycle_event(
        &self,
        session_id: &str,
        event_name: &str,
        attributes: Option<&str>,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO hook_lifecycle_events (session_id, occurred_at, event_name, attributes) \
             VALUES (?1,?2,?3,?4)",
            params![session_id, now, event_name, attributes],
        )?;
        Ok(())
    }

    // ── Hook event query methods ──────────────────────────────────────────────

    pub fn get_tool_events(
        &self,
        session_id: &str,
        event_filter: Option<&str>,
    ) -> Result<Vec<HookToolEvent>> {
        let (sql, p2) = if let Some(ev) = event_filter {
            (
                "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
                 error_message,is_interrupt,tool_use_id,cwd \
                 FROM hook_tool_events WHERE session_id=?1 AND hook_event=?2 ORDER BY occurred_at"
                    .to_string(),
                Some(ev.to_string()),
            )
        } else {
            (
                "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
                 error_message,is_interrupt,tool_use_id,cwd \
                 FROM hook_tool_events WHERE session_id=?1 ORDER BY occurred_at"
                    .to_string(),
                None,
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(ref ev) = p2 {
            stmt.query_map(params![session_id, ev], hook_tool_event_from_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![session_id], hook_tool_event_from_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    pub fn get_tool_failures(&self, session_id: &str) -> Result<Vec<HookToolEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
             error_message,is_interrupt,tool_use_id,cwd \
             FROM hook_tool_events WHERE session_id=?1 AND hook_event='PostToolUseFailure' \
             ORDER BY occurred_at",
        )?;
        let rows = stmt
            .query_map(params![session_id], hook_tool_event_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_subagent_events(&self, session_id: &str) -> Result<Vec<HookSubagentEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,session_id,occurred_at,hook_event,agent_type,agent_name,cwd \
             FROM hook_subagent_events WHERE session_id=?1 ORDER BY occurred_at",
        )?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(HookSubagentEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    hook_event: row.get(3)?,
                    agent_type: row.get(4)?,
                    agent_name: row.get(5)?,
                    cwd: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_compaction_events(&self, session_id: &str) -> Result<Vec<HookCompactionEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,session_id,occurred_at,compaction_trigger \
             FROM hook_compaction_events WHERE session_id=?1 ORDER BY occurred_at",
        )?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(HookCompactionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    compaction_trigger: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_permission_events(&self, session_id: &str) -> Result<Vec<HookPermissionEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,session_id,occurred_at,tool_name,tool_input,cwd \
             FROM hook_permission_events WHERE session_id=?1 ORDER BY occurred_at",
        )?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(HookPermissionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    tool_name: row.get(3)?,
                    tool_input: row.get(4)?,
                    cwd: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_lifecycle_events(
        &self,
        session_id: &str,
        event_filter: Option<&str>,
    ) -> Result<Vec<HookLifecycleEvent>> {
        let (sql, p2) = if let Some(ev) = event_filter {
            (
                "SELECT id,session_id,occurred_at,event_name,attributes \
                 FROM hook_lifecycle_events WHERE session_id=?1 AND event_name=?2 \
                 ORDER BY occurred_at"
                    .to_string(),
                Some(ev.to_string()),
            )
        } else {
            (
                "SELECT id,session_id,occurred_at,event_name,attributes \
                 FROM hook_lifecycle_events WHERE session_id=?1 ORDER BY occurred_at"
                    .to_string(),
                None,
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(ref ev) = p2 {
            stmt.query_map(params![session_id, ev], |row| {
                Ok(HookLifecycleEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    event_name: row.get(3)?,
                    attributes: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![session_id], |row| {
                Ok(HookLifecycleEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    event_name: row.get(3)?,
                    attributes: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    // ── Global hook queries (no session_id filter) ─────────────────

    pub fn get_global_tool_events(
        &self,
        event_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<HookToolEvent>> {
        let (sql, p2) = if let Some(ev) = event_filter {
            (
                format!(
                    "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
                     error_message,is_interrupt,tool_use_id,cwd \
                     FROM hook_tool_events WHERE hook_event=?1 ORDER BY occurred_at DESC LIMIT {limit}"
                ),
                Some(ev.to_string()),
            )
        } else {
            (
                format!(
                    "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
                     error_message,is_interrupt,tool_use_id,cwd \
                     FROM hook_tool_events ORDER BY occurred_at DESC LIMIT {limit}"
                ),
                None,
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(ref ev) = p2 {
            stmt.query_map(params![ev], hook_tool_event_from_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], hook_tool_event_from_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    pub fn get_global_tool_failures(&self, limit: usize) -> Result<Vec<HookToolEvent>> {
        let sql = format!(
            "SELECT id,session_id,occurred_at,hook_event,tool_name,tool_input,tool_result,\
             error_message,is_interrupt,tool_use_id,cwd \
             FROM hook_tool_events WHERE error_message IS NOT NULL \
             ORDER BY occurred_at DESC LIMIT {limit}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], hook_tool_event_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_global_subagent_events(&self, limit: usize) -> Result<Vec<HookSubagentEvent>> {
        let sql = format!(
            "SELECT id,session_id,occurred_at,hook_event,agent_type,agent_name,cwd \
             FROM hook_subagent_events ORDER BY occurred_at DESC LIMIT {limit}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HookSubagentEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    hook_event: row.get(3)?,
                    agent_type: row.get(4)?,
                    agent_name: row.get(5)?,
                    cwd: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_global_compaction_events(&self, limit: usize) -> Result<Vec<HookCompactionEvent>> {
        let sql = format!(
            "SELECT id,session_id,occurred_at,compaction_trigger \
             FROM hook_compaction_events ORDER BY occurred_at DESC LIMIT {limit}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HookCompactionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    compaction_trigger: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_global_permission_events(
        &self,
        limit: usize,
    ) -> Result<Vec<HookPermissionEvent>> {
        let sql = format!(
            "SELECT id,session_id,occurred_at,tool_name,tool_input,cwd \
             FROM hook_permission_events ORDER BY occurred_at DESC LIMIT {limit}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HookPermissionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    tool_name: row.get(3)?,
                    tool_input: row.get(4)?,
                    cwd: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_global_lifecycle_events(
        &self,
        event_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<HookLifecycleEvent>> {
        let (sql, p2) = if let Some(ev) = event_filter {
            (
                format!(
                    "SELECT id,session_id,occurred_at,event_name,attributes \
                     FROM hook_lifecycle_events WHERE event_name=?1 \
                     ORDER BY occurred_at DESC LIMIT {limit}"
                ),
                Some(ev.to_string()),
            )
        } else {
            (
                format!(
                    "SELECT id,session_id,occurred_at,event_name,attributes \
                     FROM hook_lifecycle_events ORDER BY occurred_at DESC LIMIT {limit}"
                ),
                None,
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(ref ev) = p2 {
            stmt.query_map(params![ev], |row| {
                Ok(HookLifecycleEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    event_name: row.get(3)?,
                    attributes: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(HookLifecycleEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    occurred_at: row.get(2)?,
                    event_name: row.get(3)?,
                    attributes: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    pub fn get_activity_summary(&self) -> Result<crate::server::routes::hooks::HookActivitySummary> {
        let total_tool: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM hook_tool_events", [], |r| r.get(0))
            .unwrap_or(0);
        let total_subagent: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM hook_subagent_events", [], |r| r.get(0))
            .unwrap_or(0);
        let total_lifecycle: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM hook_lifecycle_events", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        let total_permission: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM hook_permission_events", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        // Top tools by event count
        let mut stmt = self.conn.prepare(
            "SELECT tool_name, COUNT(*) as cnt FROM hook_tool_events \
             WHERE tool_name IS NOT NULL GROUP BY tool_name ORDER BY cnt DESC LIMIT 10",
        )?;
        let tool_counts: Vec<(String, usize)> = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Recent errors (last 24h)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let cutoff = now - 86400;
        let recent_errors: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM hook_tool_events WHERE error_message IS NOT NULL AND occurred_at > ?1",
                params![cutoff],
                |r| r.get(0),
            )
            .unwrap_or(0);

        Ok(crate::server::routes::hooks::HookActivitySummary {
            total_tool_events: total_tool as usize,
            total_subagent_events: total_subagent as usize,
            total_lifecycle_events: total_lifecycle as usize,
            total_permission_events: total_permission as usize,
            tool_event_counts: tool_counts,
            recent_errors: recent_errors as usize,
        })
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Aggregated OTLP token/cost summary for a single session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OtelSessionSummary {
    pub session_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
    pub api_requests: usize,
}

/// Aggregated OTLP token/cost summary across all sessions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OtelGlobalSummary {
    pub total_sessions: usize,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
    pub api_requests: usize,
}

fn hook_tool_event_from_row(row: &rusqlite::Row) -> rusqlite::Result<HookToolEvent> {
    Ok(HookToolEvent {
        id: row.get(0)?,
        session_id: row.get(1)?,
        occurred_at: row.get(2)?,
        hook_event: row.get(3)?,
        tool_name: row.get(4)?,
        tool_input: row.get(5)?,
        tool_result: row.get(6)?,
        error_message: row.get(7)?,
        is_interrupt: row.get::<_, Option<i64>>(8)?.map(|v| v != 0),
        tool_use_id: row.get(9)?,
        cwd: row.get(10)?,
    })
}

/// Hook tool event record (PreToolUse, PostToolUse, PostToolUseFailure)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookToolEvent {
    pub id: i64,
    pub session_id: String,
    pub occurred_at: i64,
    pub hook_event: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub tool_result: Option<String>,
    pub error_message: Option<String>,
    pub is_interrupt: Option<bool>,
    pub tool_use_id: Option<String>,
    pub cwd: Option<String>,
}

/// Hook subagent event record (SubagentStart, SubagentStop)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookSubagentEvent {
    pub id: i64,
    pub session_id: String,
    pub occurred_at: i64,
    pub hook_event: String,
    pub agent_type: Option<String>,
    pub agent_name: Option<String>,
    pub cwd: Option<String>,
}

/// Hook compaction event record (PreCompact)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookCompactionEvent {
    pub id: i64,
    pub session_id: String,
    pub occurred_at: i64,
    pub compaction_trigger: Option<String>,
}

/// Hook permission event record (PermissionRequest)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookPermissionEvent {
    pub id: i64,
    pub session_id: String,
    pub occurred_at: i64,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub cwd: Option<String>,
}

/// Hook lifecycle event record (TaskCompleted, WorktreeCreate, WorktreeRemove, ConfigChange)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookLifecycleEvent {
    pub id: i64,
    pub session_id: String,
    pub occurred_at: i64,
    pub event_name: String,
    pub attributes: Option<String>,
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
            node_type: crate::parser::models::NodeType::User,
            is_sidechain: None,
            session_id: None,
            cwd: None,
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
                node_type: crate::parser::models::NodeType::User,
                is_sidechain: None,
                session_id: None,
                cwd: None,
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
                node_type: crate::parser::models::NodeType::Unknown,
                is_sidechain: None,
                session_id: None,
                cwd: None,
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
                node_type: crate::parser::models::NodeType::Unknown,
                is_sidechain: None,
                session_id: None,
                cwd: None,
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
                node_type: crate::parser::models::NodeType::Unknown,
                is_sidechain: None,
                session_id: None,
                cwd: None,
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

    #[test]
    fn test_schema_migration_deletes_stale_db() {
        // Simulate a v1.x database with an old schema version.
        // `open()` should delete it and recreate with the current schema.
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        // Create a DB with an old user_version and a dummy table layout
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE sessions (session_id TEXT PRIMARY KEY, project_name TEXT);",
            )
            .unwrap();
            conn.execute("PRAGMA user_version = 2", []).unwrap();
            // DB is closed when `conn` drops
        }

        // open() should detect the version mismatch, delete, and rebuild
        let (index, needs_reindex) = SessionIndex::open(&db_path).unwrap();
        assert!(needs_reindex, "should flag that a reindex is needed");

        // The new DB should have the current schema version
        let version: i64 = index
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // All v2 tables should exist (spot-check a few)
        let table_exists = |name: &str| -> bool {
            index
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [name],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap()
                > 0
        };
        assert!(table_exists("sessions"));
        assert!(table_exists("tool_usage"));
        assert!(table_exists("file_usage"));
        assert!(table_exists("hook_tool_events"));
        assert!(table_exists("otel_metrics"));
        assert!(table_exists("otel_logs"));
    }

    #[test]
    fn test_schema_migration_skips_current_version() {
        // A DB that already has the current schema version should NOT be deleted.
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        // Create a proper DB at the current schema version
        {
            let (index, needs_reindex) = SessionIndex::open(&db_path).unwrap();
            assert!(!needs_reindex, "fresh DB should not need reindex");

            // Insert a sentinel row so we can verify the DB was NOT deleted
            index
                .conn
                .execute(
                    "INSERT INTO sessions (session_id, project_name, file_path, file_size, modified_at, has_subagents, indexed_at) \
                     VALUES ('sentinel', 'test', '/tmp/x', 0, 0, 0, 0)",
                    [],
                )
                .unwrap();
        }

        // Reopen — should keep the existing DB intact
        let (index, needs_reindex) = SessionIndex::open(&db_path).unwrap();
        assert!(!needs_reindex);

        let count: i64 = index
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE session_id = 'sentinel'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "sentinel row should still exist");
    }

    #[test]
    fn test_schema_migration_fresh_db() {
        // A brand-new DB (version 0) should be created without triggering reindex.
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        assert!(!db_path.exists());
        let (_index, needs_reindex) = SessionIndex::open(&db_path).unwrap();
        assert!(!needs_reindex, "fresh DB should not trigger reindex");
    }
}
