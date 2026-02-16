//! Search command - deprecated in favor of TUI search
//!
//! The search functionality has been moved to the TUI (press 'Q' in any view).
//! This CLI command is kept for backwards compatibility but will be removed in a future version.

use crate::error::Result;

pub fn run(_query: String, _tools: Vec<String>, _errors: bool) -> Result<()> {
    println!("❌ The CLI search command has been replaced with an interactive TUI search.");
    println!();
    println!("To search:");
    println!("  1. Run 'hindsight' to open the TUI");
    println!("  2. Press '/' to open search (like vim/less)");
    println!("  3. Type to filter sessions in real-time");
    println!();
    println!("The new search provides:");
    println!("  • Live fuzzy matching as you type");
    println!("  • Context-aware search (projects, sessions, or session content)");
    println!("  • Preview pane showing selected results");
    println!("  • Keyboard navigation (↑/↓/Enter/Esc)");
    println!();
    println!("This CLI command will be removed in version 0.2.0");

    Ok(())
}
