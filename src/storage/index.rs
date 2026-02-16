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

    /// Initialize the database schema
    fn initialize_schema(&mut self) -> Result<()> {
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
            "#,
        )?;

        Ok(())
    }

    /// Index a single session file
    pub fn index_session(&mut self, session: &SessionFile) -> Result<()> {
        let indexed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
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
}

/// Statistics for a project
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub project_name: String,
    pub session_count: usize,
    pub total_size: u64,
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
        let _temp_dir = TempDir::new().unwrap();
        let mut index = SessionIndex::new().unwrap();

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
