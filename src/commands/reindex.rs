//! Implementation of the `reindex` command
//!
//! Syncs the session index with what's actually on disk:
//!   1. Prune sessions whose files have been deleted
//!   2. Discover and add any new sessions not yet in the index
//!   3. Re-parse every remaining session to refresh all analytics

use crate::error::Result;
use crate::storage::{discover_sessions, SessionIndex};

pub fn run(verbose: bool) -> Result<()> {
    println!("Syncing session index...\n");

    let mut index = SessionIndex::new()?;

    // ── Step 1: prune sessions that no longer exist on disk ──────────────
    print!("  [1/3] Removing stale entries... ");
    let pruned = index.prune_missing()?;
    if pruned == 0 {
        println!("none found");
    } else {
        println!("{} removed", pruned);
    }

    // ── Step 2: discover sessions on disk and add any that are new ───────
    print!("  [2/3] Scanning for new sessions... ");
    let discovered = match discover_sessions() {
        Ok(sessions) => sessions,
        Err(crate::error::HindsightError::NoSessionsFound) => {
            println!("no sessions found on disk");
            vec![]
        }
        Err(e) => return Err(e),
    };

    let existing_ids: std::collections::HashSet<String> = index
        .list_sessions()?
        .into_iter()
        .map(|s| s.session_id)
        .collect();

    let new_sessions: Vec<_> = discovered
        .iter()
        .filter(|s| !existing_ids.contains(&s.session_id))
        .collect();

    let new_count = new_sessions.len();
    if new_count == 0 {
        println!("none found");
    } else {
        println!("{} new session(s)", new_count);
        for session in &new_sessions {
            if verbose {
                println!("     + {} ({})", session.session_id, session.project_name);
            }
            index.index_session(session)?;
        }
    }

    // ── Step 3: re-parse every session for fresh analytics ───────────────
    let sessions = index.list_sessions()?;
    let total = sessions.len();

    if total == 0 {
        println!("  [3/3] No sessions to reparse.");
        println!("\nSync complete — index is empty. Run 'hindsight init' if you have Claude Code sessions.");
        return Ok(());
    }

    println!("  [3/3] Refreshing analytics for {} session(s)...", total);

    let mut updated = 0;
    let mut errors = 0;

    for (i, session) in sessions.iter().enumerate() {
        if verbose {
            println!(
                "     [{}/{}] {} ({})",
                i + 1,
                total,
                session.session_id,
                session.project_name
            );
        }

        match index.index_session(session) {
            Ok(_) => updated += 1,
            Err(e) => {
                if verbose {
                    eprintln!("     Warning: {}", e);
                }
                errors += 1;
            }
        }
    }

    // ── Summary ───────────────────────────────────────────────────────────
    println!("\nSync complete!\n");
    println!("  Summary:");
    println!("   Pruned (deleted from disk): {}", pruned);
    println!("   New sessions added:         {}", new_count);
    println!("   Analytics refreshed:        {}", updated);
    if errors > 0 {
        println!("   Parse errors:               {}", errors);
    }
    println!("\n  Total sessions in index: {}", updated);

    Ok(())
}
