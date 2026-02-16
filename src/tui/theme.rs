//! Visual theme and styling for the TUI
//!
//! Defines colors, icons, and visual constants for a polished interface.

use ratatui::style::Color;

/// Icon set for different node types (Nerd Font icons)
pub mod icons {
    pub const USER: &str = "";           // nf-fa-user
    pub const ASSISTANT: &str = "";      // nf-fa-robot
    pub const TOOL_USE: &str = "";       // nf-fa-wrench
    pub const TOOL_RESULT: &str = "";    // nf-fa-check_circle
    pub const THINKING: &str = "";       // nf-fa-lightbulb
    pub const PROGRESS: &str = "";       // nf-fa-clock
    pub const ERROR: &str = "";          // nf-fa-times_circle
    pub const SUCCESS: &str = "";        // nf-fa-check_circle
    pub const FILE: &str = "";           // nf-fa-file_code
    pub const SYSTEM: &str = "";         // nf-fa-cog
    pub const SEARCH: &str = "";         // nf-fa-search
    pub const FILTER: &str = "";         // nf-fa-filter
}

/// Box drawing characters for visual separation
pub mod box_chars {
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
    pub const DOUBLE_HORIZONTAL: &str = "═";
}

/// Enhanced color palette
pub mod colors {
    use super::Color;

    // Primary colors
    pub const USER_MSG: Color = Color::Cyan;
    pub const ASSISTANT_MSG: Color = Color::Green;
    pub const TOOL_USE: Color = Color::Yellow;
    pub const TOOL_RESULT: Color = Color::Blue;
    pub const THINKING: Color = Color::Magenta;
    pub const PROGRESS: Color = Color::Gray;

    // Accent colors
    pub const SUCCESS: Color = Color::LightGreen;
    pub const ERROR: Color = Color::LightRed;
    pub const WARNING: Color = Color::LightYellow;
    pub const INFO: Color = Color::LightCyan;

    // Syntax highlighting
    pub const JSON_KEY: Color = Color::Cyan;
    pub const JSON_STRING: Color = Color::Green;
    pub const JSON_NUMBER: Color = Color::Yellow;
    pub const JSON_BOOLEAN: Color = Color::Magenta;
    pub const JSON_NULL: Color = Color::Gray;

    // UI elements
    pub const HEADER: Color = Color::LightBlue;
    pub const LABEL: Color = Color::Cyan;
    pub const VALUE: Color = Color::White;
    pub const DIM: Color = Color::Gray;
    pub const BORDER: Color = Color::DarkGray;
}

/// Format a header with icon and title
pub fn format_header(icon: &str, title: &str) -> String {
    format!("{} {}", icon, title)
}

/// Create a separator line
pub fn separator(width: usize) -> String {
    box_chars::HORIZONTAL.repeat(width)
}

/// Create a fancy box header
pub fn box_header(title: &str, width: usize) -> String {
    let inner_width = width.saturating_sub(4);
    let title_len = title.chars().count();
    let padding = if title_len < inner_width {
        (inner_width - title_len) / 2
    } else {
        0
    };

    format!(
        "{}{}{} {} {}{}{}",
        box_chars::TOP_LEFT,
        box_chars::HORIZONTAL,
        " ".repeat(padding),
        title,
        " ".repeat(inner_width.saturating_sub(title_len + padding)),
        box_chars::HORIZONTAL,
        box_chars::TOP_RIGHT
    )
}

/// Create a box footer
pub fn box_footer(width: usize) -> String {
    format!(
        "{}{}{}",
        box_chars::BOTTOM_LEFT,
        box_chars::HORIZONTAL.repeat(width.saturating_sub(2)),
        box_chars::BOTTOM_RIGHT
    )
}
