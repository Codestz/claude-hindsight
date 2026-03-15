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

    /// Model used (short name, e.g. "sonnet-4-5")
    pub model: Option<String>,

    /// Number of errors (tool result errors + error nodes)
    pub error_count: usize,

    /// First user message preview (up to 80 chars)
    pub first_message: Option<String>,

    /// Source directory path (from config, e.g. "~/.claude/projects")
    pub source_dir: String,

    /// Comma-separated unique models used by subagents (e.g. "claude-haiku-4-5")
    pub subagent_models: Option<String>,
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
    let home = dirs::home_dir()
        .ok_or_else(|| HindsightError::Config("Could not determine home directory".to_string()))?;

    let config = crate::config::Config::load().unwrap_or_default();

    // Resolve configured directories: expand ~ and filter to those that exist.
    // Each entry carries (expanded_path, raw_config_path) for source_dir tracking.
    let claude_dirs: Vec<(PathBuf, String)> = config
        .paths
        .claude_dirs
        .iter()
        .map(|d| {
            let expanded = if let Some(stripped) = d.path.strip_prefix("~/") {
                home.join(stripped)
            } else {
                PathBuf::from(&d.path)
            };
            (expanded, d.path.clone())
        })
        .filter(|(p, _)| p.exists())
        .collect();

    let mut sessions = Vec::new();

    for (claude_dir, source_dir) in &claude_dirs {
        if !claude_dir.exists() {
            continue;
        }

        // Scan each project directory
        for project_entry in fs::read_dir(claude_dir)? {
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
                if file_path.is_file()
                    && file_path.extension().and_then(|s| s.to_str()) == Some("jsonl")
                {
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
                        model: None,
                        error_count: 0,
                        first_message: None,
                        source_dir: source_dir.clone(),
                        subagent_models: None,
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
/// Decode a project name from Claude Code's encoded directory name.
///
/// Claude Code encodes project paths by replacing `/` with `-`:
///   `-Users-codestz-Documents-PersonalProjects-claude-hindsight` → `claude-hindsight`
///   `-Users-codestz-Documents-Projects-dev-container-poc` → `dev-container-poc`
///   `-Users-codestz` → `codestz`
///   `-` → (unnamed, root scope)
///
/// Strategy: reconstruct the original path, take the last component.
fn decode_project_name(path: &Path) -> String {
    let dir_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Edge case: just "-" or empty → unnamed project
    if dir_name.is_empty() || dir_name == "-" {
        return String::new();
    }

    // If it doesn't start with '-', it's already a plain name
    if !dir_name.starts_with('-') {
        return dir_name.to_string();
    }

    // Reconstruct the original filesystem path by replacing leading `-` with `/`
    // The encoded format: `-Users-name-path-to-project` represents `/Users/name/path/to/project`
    let reconstructed = dir_name.replacen('-', "/", 1); // first `-` → `/`

    // Now we have something like `/Users-name-path-to-project`
    // We need to figure out where the actual path separators were.
    // Claude Code uses the *full absolute path* encoded with `-` for `/`.
    // Known path segments that are always single words: Users, Documents, home, var, tmp, etc.
    // The project name is the last directory component which may contain hyphens.
    //
    // Strategy: find the last known parent directory marker and take everything after it.
    let known_parents = [
        "PersonalProjects-",
        "Projects-",
        "workspace-",
        "Workspace-",
        "repos-",
        "Repos-",
        "src-",
        "dev-",
        "code-",
        "Code-",
        "github-",
        "GitHub-",
        "git-",
    ];

    for parent in &known_parents {
        if let Some(pos) = dir_name.rfind(parent) {
            let after = &dir_name[pos + parent.len()..];
            if !after.is_empty() {
                return after.to_string();
            }
        }
    }

    // Fallback: for paths like `-Users-codestz` (home dir sessions),
    // take the last segment after the last `-` that follows a known single-word segment
    // Simple heuristic: split by `-`, skip known path components, rejoin the rest
    let segments: Vec<&str> = dir_name.split('-').filter(|s| !s.is_empty()).collect();

    // Skip known prefixes: Users, username, Documents, etc.
    // Find the first segment that's NOT a common path component
    let skip_words: std::collections::HashSet<&str> = [
        "Users", "home", "var", "tmp", "opt", "Documents", "Desktop",
        "Downloads", "Library",
    ].iter().copied().collect();

    let mut project_start = 0;
    for (i, seg) in segments.iter().enumerate() {
        if skip_words.contains(seg) {
            project_start = i + 1;
        } else if i > 0 && i == project_start {
            // This is probably the username — skip one more
            project_start = i + 1;
        } else {
            break;
        }
    }

    if project_start < segments.len() {
        segments[project_start..].join("-")
    } else if !segments.is_empty() {
        // All segments were "known" — use the last one (e.g., username)
        segments.last().unwrap().to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_project_name() {
        // Standard project paths
        assert_eq!(
            decode_project_name(Path::new("-Users-codestz-Documents-PersonalProjects-claude-hindsight")),
            "claude-hindsight"
        );
        assert_eq!(
            decode_project_name(Path::new("-Users-codestz-Documents-PersonalProjects-prompt-evaluator")),
            "prompt-evaluator"
        );
        assert_eq!(
            decode_project_name(Path::new("-Users-codestz-Documents-PersonalProjects-mcpx")),
            "mcpx"
        );
        assert_eq!(
            decode_project_name(Path::new("-Users-codestz-Documents-Projects-dev-container-poc")),
            "dev-container-poc"
        );

        // Home directory sessions
        assert_eq!(
            decode_project_name(Path::new("-Users-codestz")),
            "codestz"
        );

        // Root/empty
        assert_eq!(decode_project_name(Path::new("-")), "");

        // Plain names (non-encoded)
        assert_eq!(decode_project_name(Path::new("my-project")), "my-project");
    }
}
