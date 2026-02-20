//! Event handling for TUI
//!
//! Manages keyboard input and other terminal events.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;

use crate::error::Result;

/// Terminal events
#[derive(Debug)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),

    /// Terminal resize event
    #[allow(dead_code)]
    Resize(u16, u16),
}

/// Event handler for terminal events
pub struct EventHandler {
    /// Poll timeout in milliseconds
    timeout: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(timeout_ms: u64) -> Self {
        EventHandler {
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Get the next event (blocking with timeout)
    pub fn poll(&mut self) -> Result<Event> {
        if event::poll(self.timeout)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
                _ => self.poll(), // Ignore other events
            }
        } else {
            // Timeout - return a dummy resize to keep the loop going
            Ok(Event::Resize(0, 0))
        }
    }
}
