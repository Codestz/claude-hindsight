//! Error types for Claude Hindsight
//!
//! Uses `thiserror` for structured error handling following Rust best practices.

use thiserror::Error;

/// Result type alias for Claude Hindsight operations
pub type Result<T> = std::result::Result<T, HindsightError>;

#[derive(Error, Debug)]
pub enum HindsightError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error at line {line}: {message}")]
    JsonParse { line: usize, message: String },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File watcher error: {0}")]
    FileWatcher(String),

    #[error("No Claude Code sessions found")]
    NoSessionsFound,

    #[error("Invalid session format: {0}")]
    InvalidSession(String),
}

impl From<serde_json::Error> for HindsightError {
    fn from(err: serde_json::Error) -> Self {
        HindsightError::JsonParse {
            line: err.line(),
            message: err.to_string(),
        }
    }
}
