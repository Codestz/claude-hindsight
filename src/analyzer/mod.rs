//! Execution tree analysis
//!
//! Builds hierarchical tree structures from flat JSONL execution nodes.

pub mod simple_tree;
pub mod smart_label;
pub mod tree;

pub use simple_tree::build_simple_tree;
pub use tree::TreeNode;
