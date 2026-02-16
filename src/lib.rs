//! Claude Hindsight library
//!
//! Provides JSONL parsing and session analysis for Claude Code transcripts.

pub mod error;
pub mod parser;

pub use error::{HindsightError, Result};
pub use parser::{ExecutionNode, Session, parse_session};
