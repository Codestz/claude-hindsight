//! Interactive terminal UI
//!
//! Provides a lazygit-style terminal interface for exploring Claude Code sessions.

pub mod app;
pub mod events;
pub mod projects_view;
pub mod render;
pub mod router;
pub mod search;
pub mod sessions_view;
pub mod theme;
pub mod ui;

pub use app::App;
pub use events::{Event, EventHandler};

use crate::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Initialize the terminal for TUI mode
pub fn init() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

/// Run the TUI application
pub fn run(app: &mut App) -> Result<()> {
    let mut terminal = init()?;
    let mut event_handler = EventHandler::new(250);

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event_handler.next()? {
            app.handle_key(key)?;
        }

        if app.should_quit {
            break;
        }
    }

    restore()?;
    Ok(())
}
