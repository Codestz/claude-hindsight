//! Implementation of the `stats` command
//!
//! Shows quick statistics for a Claude Code session.

use crate::error::{HindsightError, Result};
use crate::parser::parse_session;
use crate::storage::SessionIndex;
use chrono::{DateTime, Local};

pub fn run(session_id: String) -> Result<()> {
    let index = SessionIndex::new()?;

    // Find session by ID (supports prefix matching)
    let session_file = index
        .find_by_id(&session_id)?
        .ok_or_else(|| HindsightError::SessionNotFound(session_id.clone()))?;

    println!("📊 Session Statistics\n");
    println!("   ID: {}", session_file.session_id);
    println!("   Project: {}", session_file.project_name);

    let timestamp = DateTime::from_timestamp(session_file.modified_at, 0)
        .map(|dt| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "Unknown".to_string());

    println!("   Modified: {}", timestamp);
    println!("   File size: {} KB", session_file.file_size / 1024);

    if session_file.has_subagents {
        println!("   Subagents: Yes 🌲");
    }

    // Parse the session to get detailed stats
    println!("\n🔍 Parsing session...");
    let session = parse_session(&session_file.path)?;

    println!("\n📈 Execution Summary:");
    println!("   Total nodes: {}", session.nodes.len());
    println!("   Tool calls: {}", session.total_tools);
    println!("   Errors: {}", session.error_count);

    if let (Some(start), Some(end)) = (session.start_time, session.end_time) {
        let duration_ms = end - start;
        let duration_secs = duration_ms / 1000;
        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;

        println!("   Duration: {}m {}s", minutes, seconds);
    }

    println!("\n💰 Token Usage:");
    println!("   Total tokens: {}", session.total_tokens);
    println!("   Estimated cost: ${:.4}", session.estimated_cost);

    // Count tool types
    let mut tool_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for node in &session.nodes {
        if let Some(ref tool_use) = node.tool_use {
            *tool_counts.entry(tool_use.name.clone()).or_insert(0) += 1;
        }
    }

    if !tool_counts.is_empty() {
        println!("\n🔧 Tool Usage:");
        let mut sorted_tools: Vec<_> = tool_counts.iter().collect();
        sorted_tools.sort_by(|a, b| b.1.cmp(a.1));

        for (tool, count) in sorted_tools.iter().take(10) {
            println!("   {} - {} calls", tool, count);
        }
    }

    Ok(())
}
