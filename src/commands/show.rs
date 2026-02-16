//! Implementation of the `show` command
//!
//! Displays execution tree for a Claude Code session.

use crate::error::{HindsightError, Result};
use crate::parser::parse_session;
use crate::storage::SessionIndex;
use crate::tui::{App, run as run_tui};

pub fn run(session_id: String, dashboard: bool, _port: u16) -> Result<()> {
    if dashboard {
        println!("🌐 Web dashboard not yet implemented");
        println!("💡 Use without --dashboard flag to view in terminal\n");
        return Ok(());
    }

    // Find session
    let index = SessionIndex::new()?;
    let session_file = index
        .find_by_id(&session_id)?
        .ok_or_else(|| HindsightError::SessionNotFound(session_id.clone()))?;

    // Parse session
    let session = parse_session(&session_file.path)?;

    // Launch TUI
    let mut app = App::new(session);
    run_tui(&mut app)?;

    Ok(())
}
