//! Session file discovery
//!
//! Scans Claude Code project directories for JSONL transcript files.

use crate::error::{HindsightError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a discovered session file
#[derive(Debug, Clone)]
pub struct SessionFile {
    /// Full path to the JSONL file
    pub path: PathBuf,

    /// Session ID (extracted from filename)
    pub session_id: String,

    /// Project name (decoded from directory name)
    pub project_name: String,

    /// File size in bytes
    pub file_size: u64,

    /// Session creation timestamp (seconds since epoch, from first node's timestamp)
    pub created_at: i64,

    /// Last modified timestamp (seconds since epoch)
    pub modified_at: i64,

    /// Whether this session has subagents (folder with subagents/ directory)
    pub has_subagents: bool,

    /// Total tokens (input + output) for the session
    pub total_tokens: u64,

    /// Estimated cost in USD
    pub estimated_cost: f64,

    /// Model used (short name, e.g. "sonnet-4-5")
    pub model: Option<String>,

    /// Number of errors (tool result errors + error nodes)
    pub error_count: usize,

    /// First user message preview (up to 80 chars)
    pub first_message: Option<String>,
}

/// Discover all Claude Code sessions in configured directories
///
/// Scans `~/.claude/projects/` for session JSONL files.
///
/// # Returns
///
/// Returns a vector of `SessionFile` structs representing all discovered sessions.
///
/// # Errors
///
/// Returns `HindsightError::NoSessionsFound` if no sessions are discovered.
pub fn discover_sessions() -> Result<Vec<SessionFile>> {
    let home = dirs::home_dir().ok_or_else(|| {
        HindsightError::Config("Could not determine home directory".to_string())
    })?;

    let claude_dirs = vec![
        home.join(".claude/projects"),
    ];

    let mut sessions = Vec::new();

    for claude_dir in claude_dirs {
        if !claude_dir.exists() {
            continue;
        }

        // Scan each project directory
        for project_entry in fs::read_dir(&claude_dir)? {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            // Extract project name from directory name
            let project_name = decode_project_name(&project_path);

            // Find all .jsonl files in this project
            for file_entry in fs::read_dir(&project_path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                // Check if it's a .jsonl file (not a directory)
                if file_path.is_file() &&
                   file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {

                    let metadata = fs::metadata(&file_path)?;
                    let session_id = file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let modified_at = metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    // Check if there's a matching directory with subagents
                    let subagents_dir = project_path.join(&session_id).join("subagents");
                    let has_subagents = subagents_dir.exists() && subagents_dir.is_dir();

                    sessions.push(SessionFile {
                        path: file_path,
                        session_id,
                        project_name: project_name.clone(),
                        file_size: metadata.len(),
                        created_at: modified_at, // refined during indexing from first node timestamp
                        modified_at,
                        has_subagents,
                        total_tokens: 0,
                        estimated_cost: 0.0,
                        model: None,
                        error_count: 0,
                        first_message: None,
                    });
                }
            }
        }
    }

    if sessions.is_empty() {
        return Err(HindsightError::NoSessionsFound);
    }

    // Sort by modification time (newest first)
    sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(sessions)
}

/// Decode project name from directory path
///
/// Converts `-Users-ediazestrada-Documents-Projects-experiment` to `experiment`
fn decode_project_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Try to extract the last meaningful part
            if s.starts_with('-') {
                s.split('-').last().unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_project_name() {
        let path = Path::new("-Users-ediazestrada-Documents-Projects-experiment");
        assert_eq!(decode_project_name(path), "experiment");

        let path = Path::new("my-project");
        assert_eq!(decode_project_name(path), "my-project");
    }
}
