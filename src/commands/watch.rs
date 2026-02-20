//! Implementation of the `watch` command

use crate::error::{HindsightError, Result};
use crate::storage::SessionIndex;
use crate::watcher::stream_renderer::{render_node, render_stats, WatchStats};
use crate::watcher::SessionWatcher;
use std::time::Duration;

pub fn run(session_id: Option<String>, dashboard: bool) -> Result<()> {
    if dashboard {
        println!("Dashboard mode not yet implemented.");
        println!("Use without --dashboard flag to watch in terminal.");
        return Ok(());
    }

    // Find the session to watch
    let index = SessionIndex::new()?;
    let session_file = match session_id {
        Some(id) => {
            // Find by specific ID
            index
                .find_by_id(&id)?
                .ok_or_else(|| HindsightError::SessionNotFound(id.clone()))?
        }
        None => {
            // Auto-detect most recent session
            println!("  Auto-detecting current session...\n");
            index
                .get_latest()?
                .ok_or_else(|| HindsightError::NoSessionsFound)?
        }
    };

    println!("  Watching session: {}", session_file.session_id);
    println!("  Project: {}", session_file.project_name);
    println!("  File: {}\n", session_file.path.display());
    println!("{}", "=".repeat(80));

    // Create watcher
    let mut watcher = SessionWatcher::new(&session_file.path)?;
    let mut stats = WatchStats::default();

    // Read existing content first
    let existing_nodes = watcher.read_new_nodes()?;

    if existing_nodes.is_empty() {
        println!("  No existing nodes in session.\n");
    } else {
        println!("  Loaded {} existing nodes\n", existing_nodes.len());

        // Show a preview of existing nodes (first 5 and last 5)
        let show_count = 5;
        if existing_nodes.len() <= show_count * 2 {
            // Show all nodes if there are fewer than 10
            for node in &existing_nodes {
                render_node(node);
                stats.update(node);
            }
        } else {
            // Show first 5
            for node in existing_nodes.iter().take(show_count) {
                render_node(node);
                stats.update(node);
            }

            println!(
                "\n  ... ({} nodes omitted) ...\n",
                existing_nodes.len() - show_count * 2
            );

            // Show last 5
            for node in existing_nodes
                .iter()
                .skip(existing_nodes.len() - show_count)
            {
                render_node(node);
                stats.update(node);
            }
        }

        println!();
        render_stats(&stats);
    }

    println!("\n  Watching for new updates... (Press Ctrl+C to stop)\n");

    // Main watch loop
    loop {
        // Wait for file changes (with 1 second timeout to allow periodic checking)
        if watcher.wait_for_changes(Duration::from_secs(1)) {
            // Small delay to ensure write is complete
            std::thread::sleep(Duration::from_millis(100));

            // Read new nodes
            match watcher.poll() {
                Ok(new_nodes) => {
                    if !new_nodes.is_empty() {
                        for node in &new_nodes {
                            render_node(node);
                            stats.update(node);
                        }
                        println!();
                        render_stats(&stats);
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading new nodes: {}", e);
                }
            }
        }

        // Check if file still exists
        if !session_file.path.exists() {
            println!("\n  Session file no longer exists. Exiting.");
            break;
        }
    }

    Ok(())
}
