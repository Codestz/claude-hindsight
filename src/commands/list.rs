//! Implementation of the `list` command
//!
//! Lists all Claude Code sessions with filtering options.

use crate::error::Result;
use crate::storage::SessionIndex;
use chrono::{DateTime, Local};

pub fn run(
    project: Option<String>,
    errors: bool,
    last: Option<usize>,
    today: bool,
    with_subagents: bool,
) -> Result<()> {
    let index = SessionIndex::new()?;

    // Get sessions (filtered by project if specified)
    let mut sessions = if let Some(ref proj) = project {
        index.find_by_project(proj)?
    } else {
        index.list_sessions()?
    };

    // Filter by date if --today is specified
    if today {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let today_start = now - (now % 86400); // Start of today (midnight)

        sessions.retain(|s| s.modified_at >= today_start);
    }

    // Filter by subagents if --with-subagents is specified
    if with_subagents {
        sessions.retain(|s| s.has_subagents);
    }

    // Limit to last N sessions if specified
    if let Some(n) = last {
        sessions.truncate(n);
    }

    // Show header
    if sessions.is_empty() {
        println!("No sessions found.");
        println!("\n  Run 'hindsight init' to discover sessions");
        return Ok(());
    }

    println!("  Claude Code Sessions\n");

    if let Some(ref proj) = project {
        println!("   Project: {}\n", proj);
    }

    // Display sessions
    for session in &sessions {
        let timestamp = DateTime::from_timestamp(session.modified_at, 0)
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let size_kb = session.file_size / 1024;
        let subagents_icon = if session.has_subagents { "  " } else { "   " };

        println!(
            "{} {} | {} | {} | {} KB",
            subagents_icon,
            &session.session_id[..8],
            session.project_name,
            timestamp,
            size_kb
        );
    }

    println!("\n  Total: {} session(s)", sessions.len());

    if errors {
        println!("\n  Error filtering not yet implemented");
    }

    Ok(())
}
