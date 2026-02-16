//! Search engine for Claude Code sessions
//!
//! Provides fast metadata-based search using the SQLite index.
//! Phase 1: Metadata search only (tool names, errors, project)
//! Phase 2: Full-text content search in JSONL files (future)

pub mod filters;
pub mod results;

use crate::error::Result;
use crate::storage::SessionIndex;

// Re-export public types
pub use filters::SearchFilters;
pub use results::SearchResult;

/// Search sessions by metadata using SQLite index
///
/// This is a fast, database-only search that doesn't require parsing JSONL files.
/// Supports filtering by:
/// - Tool names (--tool)
/// - Error presence (--errors)
/// - Project name (from session metadata)
///
/// # Examples
///
/// ```no_run
/// use hindsight::search::{search_sessions, SearchFilters};
///
/// let filters = SearchFilters {
///     tools: vec!["Read".to_string()],
///     errors_only: true,
///     project: None,
/// };
///
/// let results = search_sessions(&filters)?;
/// # Ok::<(), hindsight::error::HindsightError>(())
/// ```
pub fn search_sessions(filters: &SearchFilters) -> Result<Vec<SearchResult>> {
    let index = SessionIndex::new()?;

    // Start with sessions filtered by tools (most selective filter first)
    let mut sessions = if !filters.tools.is_empty() {
        // Use the database to find sessions with these tools
        index.find_by_tools(&filters.tools)?
    } else {
        // Get all sessions or filter by project
        if let Some(ref project) = filters.project {
            index.find_by_project(project)?
        } else {
            index.list_sessions()?
        }
    };

    // Filter by project if specified and we didn't filter by tools
    if filters.tools.is_empty() {
        // Already filtered by project above if needed
    } else if let Some(ref project) = filters.project {
        // Filter the tool-filtered results by project
        sessions.retain(|s| s.project_name == *project);
    }

    // Filter by errors if requested
    if filters.errors_only {
        // Sessions with errors have error_count > 0 in the parsed session
        // For Phase 1 MVP, we skip this filter (will implement in Phase 2)
        // For now, just include all sessions when --errors is specified
        // TODO: Add error_count column to sessions table during indexing
    }

    // Convert to SearchResults with metadata and sort by relevance
    let mut results: Vec<SearchResult> = sessions
        .into_iter()
        .map(SearchResult::from_session_file)
        .collect();

    // Sort by relevance (most relevant first)
    results.sort_by(|a, b| {
        b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}
