//! Claude Hindsight library
//!
//! Provides JSONL parsing and session analysis for Claude Code transcripts.

pub mod analyzer;
pub mod api;
pub mod error;
pub mod parser;
pub mod storage;
pub mod tui;

pub use analyzer::{TreeNode, build_simple_tree};
pub use error::{HindsightError, Result};
pub use parser::{ExecutionNode, Session, parse_session};
pub use storage::{SessionFile, SessionIndex, discover_sessions, initialize_index, refresh_index};
