//! JSONL transcript parser implementation
//!
//! Parses Claude Code transcript files (JSONL format) into structured data.
//! Follows Rust best practices:
//! - Uses Result<T, E> for error handling
//! - Borrows where possible to avoid clones
//! - Iterator-based for memory efficiency

use crate::error::{HindsightError, Result};
use super::models::{ExecutionNode, Session};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse a Claude Code JSONL transcript file into a Session
///
/// # Arguments
///
/// * `path` - Path to the .jsonl transcript file
///
/// # Returns
///
/// Returns a `Session` containing all parsed execution nodes with metadata.
///
/// # Errors
///
/// Returns `HindsightError` if:
/// - File cannot be read
/// - JSONL format is invalid
/// - JSON parsing fails
///
/// # Example
///
/// ```ignore
/// use hindsight::parser::parse_session;
/// use std::path::Path;
///
/// let session = parse_session(Path::new("session.jsonl"))?;
/// println!("Found {} tools", session.total_tools);
/// ```
pub fn parse_session(path: &Path) -> Result<Session> {
    let file = File::open(path)?;

    // Estimate initial capacity from file size to reduce reallocations
    let file_metadata = file.metadata()?;
    let file_size = file_metadata.len();
    let estimated_lines = (file_size / 500).max(100) as usize; // ~500 bytes per line

    let reader = BufReader::new(file);
    let mut nodes = Vec::with_capacity(estimated_lines);

    // Parse JSONL line by line
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON line into ExecutionNode
        match serde_json::from_str::<ExecutionNode>(&line) {
            Ok(node) => nodes.push(node),
            Err(e) => {
                return Err(HindsightError::JsonParse {
                    line: line_num + 1,
                    message: e.to_string(),
                });
            }
        }
    }

    // Extract session ID from filename or first node
    let session_id = extract_session_id(path)?;

    // Get absolute path for display
    let file_path = path.canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    Ok(Session::new(session_id, file_path, nodes))
}

/// Extract session ID from file path or content
fn extract_session_id(path: &Path) -> Result<String> {
    // Try to get session ID from filename
    if let Some(file_name) = path.file_stem() {
        if let Some(name) = file_name.to_str() {
            return Ok(name.to_string());
        }
    }

    Err(HindsightError::InvalidSession(
        "Could not extract session ID from path".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_empty_file() {
        let file = NamedTempFile::new().unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 0);
    }

    #[test]
    fn test_parse_user_message() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"Hello"}}}}"#
        ).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        assert_eq!(session.nodes[0].node_type, "user");
    }

    #[test]
    fn test_parse_tool_use() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"tool_use","tool_use":{{"name":"Read","input":{{"file_path":"test.txt"}}}}}}"#
        ).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        assert!(session.nodes[0].tool_use.is_some());
        assert_eq!(session.total_tools, 1);
    }

    #[test]
    fn test_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{invalid json").unwrap();

        let result = parse_session(file.path());
        assert!(result.is_err());
    }
}
