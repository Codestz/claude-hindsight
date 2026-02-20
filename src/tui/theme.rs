//! Visual theme and styling for the TUI
//!
//! Defines colors and icons used in the interface.

use ratatui::style::Color;

/// Icon set for different node types (Nerd Font icons)
pub mod icons {
    pub const USER: &str = ""; // nf-fa-user
    pub const TOOL_USE: &str = ""; // nf-fa-wrench
    pub const TOOL_RESULT: &str = ""; // nf-fa-check_circle
    pub const THINKING: &str = ""; // nf-fa-lightbulb
    pub const PROGRESS: &str = ""; // nf-fa-clock
}

/// Color palette for TUI elements
pub mod colors {
    use super::Color;

    pub const USER_MSG: Color = Color::Cyan;
    pub const TOOL_USE: Color = Color::Yellow;
    pub const TOOL_RESULT: Color = Color::Blue;
    pub const THINKING: Color = Color::Magenta;
    pub const PROGRESS: Color = Color::Gray;
}
