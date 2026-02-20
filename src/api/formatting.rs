//! Presentation configuration for different output formats
//!
//! Separates presentation logic from data structures.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationConfig {
    pub color_scheme: ColorScheme,
    pub icon_style: IconStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IconStyle {
    NerdFont, // Terminal Nerd Fonts
    Unicode,  // Unicode emoji
    Ascii,    // ASCII fallback
    None,     // No icons
}

impl Default for PresentationConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Dark,
            icon_style: IconStyle::NerdFont,
        }
    }
}
