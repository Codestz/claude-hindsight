//! Claude Hindsight library
//!
//! Provides JSONL parsing and session analysis for Claude Code transcripts.

pub mod analyzer;
pub mod api;
pub mod config;
pub mod error;
pub mod parser;
pub mod search;
pub mod server;
pub mod storage;
pub mod tui;
pub mod watcher;

pub use analyzer::{TreeNode, build_simple_tree};
pub use error::{HindsightError, Result};
pub use parser::{ExecutionNode, Session, parse_session};
pub use storage::{SessionFile, SessionIndex, discover_sessions, initialize_index, refresh_index};
