//! Hidden command: `claude-hindsight hook-index`
//!
//! Called automatically by Claude Code via lifecycle hooks (Stop / SessionEnd).
//! Reads the hook JSON payload from stdin, extracts the session path, and
//! indexes that single file into the Hindsight database.
//!
//! Example invocation (performed by Claude Code, not users):
//!   echo '<hook-json>' | claude-hindsight hook-index

use crate::error::Result;
use std::io::{self, BufRead};

/// JSON shape emitted by Claude Code hook payloads (subset we care about).
#[derive(serde::Deserialize)]
struct HookPayload {
    transcript_path: Option<String>,
    session_id: Option<String>,
}

pub fn run() -> Result<()> {
    // Read all of stdin
    let stdin = io::stdin();
    let mut raw = String::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                raw.push_str(&l);
                raw.push('\n');
            }
            Err(_) => break,
        }
    }

    let payload: HookPayload = match serde_json::from_str(raw.trim()) {
        Ok(p) => p,
        Err(_) => {
            // Malformed or empty payload — exit silently (never block Claude Code)
            return Ok(());
        }
    };

    let transcript_path = match payload.transcript_path {
        Some(p) => p,
        None => return Ok(()),
    };

    let path = std::path::PathBuf::from(&transcript_path);
    if !path.exists() {
        return Ok(());
    }

    // Build a minimal SessionFile from the path so we can call index_session()
    let metadata = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    let session_id = payload.session_id.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Derive project_name from parent directory name (same logic as discovery)
    let project_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| {
            if s.starts_with('-') {
                s.split('-').next_back().unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let session_file = crate::storage::SessionFile {
        path,
        session_id,
        project_name,
        file_size: metadata.len(),
        created_at: modified_at,
        modified_at,
        has_subagents: false,
        model: None,
        error_count: 0,
        first_message: None,
        source_dir: String::new(),
        subagent_models: None,
    };

    // Index silently; never surface errors to Claude Code
    if let Ok(mut index) = crate::storage::SessionIndex::new() {
        let _ = index.index_session(&session_file);
    }

    Ok(())
}
