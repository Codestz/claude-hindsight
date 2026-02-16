//! API layer for web dashboard
//!
//! Provides JSON-serializable response types and formatting utilities
//! that are independent of terminal rendering.

pub mod responses;
pub mod formatting;

pub use responses::{NodeResponse, TreeResponse, SessionStatsResponse};
pub use formatting::{PresentationConfig, ColorScheme, IconStyle};
