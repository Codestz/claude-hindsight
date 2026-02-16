//! Session storage and indexing
//!
//! Discovers Claude Code sessions and maintains a fast SQLite index.

pub mod discovery;
pub mod index;

pub use discovery::{discover_sessions, SessionFile};
pub use index::SessionIndex;

use crate::error::Result;

/// Initialize or refresh the session index
///
/// Discovers all sessions and indexes them in SQLite for fast lookups.
pub fn initialize_index() -> Result<usize> {
    let sessions = discover_sessions()?;
    let mut index = SessionIndex::new()?;
    let count = index.index_all(&sessions)?;
    Ok(count)
}

/// Refresh the index and remove missing sessions
pub fn refresh_index() -> Result<(usize, usize)> {
    let mut index = SessionIndex::new()?;
    let removed = index.prune_missing()?;

    let sessions = discover_sessions()?;
    let indexed = index.index_all(&sessions)?;

    Ok((indexed, removed))
}
