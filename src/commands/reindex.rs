//! Implementation of the `reindex` command
//!
//! Reindexes all sessions to populate the tool_usage analytics table.
//! This is useful after upgrading to a version with the new SQLite schema.

use crate::error::Result;
use crate::storage::SessionIndex;

pub fn run(verbose: bool) -> Result<()> {
    println!("🔄 Reindexing sessions...\n");

    let mut index = SessionIndex::new()?;

    // Get all sessions from the database
    let sessions = index.list_sessions()?;
    let total = sessions.len();

    if total == 0 {
        println!("   No sessions found to reindex.");
        println!("\n   Run 'hindsight init' to discover and index sessions first.");
        return Ok(());
    }

    println!("   Found {} session(s) to reindex", total);

    if verbose {
        println!("\n   Progress:");
    }

    let mut reindexed = 0;
    let mut errors = 0;

    for (i, session) in sessions.iter().enumerate() {
        if verbose {
            println!(
                "   [{}/{}] {} ({})",
                i + 1,
                total,
                session.session_id,
                session.project_name
            );
        }

        // Re-index the session (this will update tool_usage table)
        match index.index_session(session) {
            Ok(_) => reindexed += 1,
            Err(e) => {
                if verbose {
                    eprintln!("      ⚠️  Error: {}", e);
                }
                errors += 1;
            }
        }
    }

    println!("\n✅ Reindexing complete!\n");
    println!("   Summary:");
    println!("   • Successfully reindexed: {}", reindexed);

    if errors > 0 {
        println!("   • Errors: {}", errors);
    }

    println!("\n   The tool_usage analytics table is now populated.");
    println!("   Analytics queries (hindsight list, stats) will now be faster!");

    Ok(())
}
