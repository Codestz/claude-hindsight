//! Claude Hindsight library
//!
//! Provides JSONL parsing and session analysis for Claude Code transcripts.

pub mod agents;
pub mod analyzer;
pub mod api;
pub mod config;
pub mod error;
pub mod otel;
pub mod parser;
pub mod pricing;
pub mod search;
pub mod server;
pub mod storage;
pub mod tui;
pub mod watcher;

pub use analyzer::{build_simple_tree, TreeNode};
pub use error::{HindsightError, Result};
pub use parser::{parse_session, ExecutionNode, Session};
pub use storage::{discover_sessions, initialize_index, refresh_index, SessionFile, SessionIndex};
