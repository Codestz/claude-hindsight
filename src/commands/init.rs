//! Implementation of the `init` command
//!
//! Discovers all Claude Code sessions and indexes them for fast lookups.

use crate::error::Result;
use crate::storage::{discover_sessions, initialize_index};

pub fn run(enable_otel: bool) -> Result<()> {
    println!("  Discovering Claude Code sessions...\n");

    // Discover all sessions
    let sessions = discover_sessions()?;
    println!("   Found {} session(s)\n", sessions.len());

    // Show discovery locations
    println!("  Scanned directories:");
    println!("   ~/.claude/projects/");

    // Index sessions
    println!("  Building session index...");
    let indexed = initialize_index()?;
    println!("   Indexed {} session(s)\n", indexed);

    // Show summary
    println!("  Initialization complete!\n");
    println!("  Summary:");

    // Group by project
    let mut projects: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for session in &sessions {
        *projects.entry(session.project_name.clone()).or_insert(0) += 1;
    }

    for (project, count) in projects.iter() {
        println!("   {} - {} session(s)", project, count);
    }

    println!("\n  Next steps:");
    println!("   hindsight list          - List all sessions");
    println!("   hindsight show <id>     - Analyze a session");
    println!("   hindsight watch         - Monitor current session");

    if enable_otel {
        println!("\n  OpenTelemetry integration not yet implemented");
    }

    Ok(())
}
