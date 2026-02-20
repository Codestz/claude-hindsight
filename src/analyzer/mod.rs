//! Execution tree analysis
//!
//! Builds hierarchical tree structures from flat JSONL execution nodes.

pub mod session_analytics;
pub mod simple_tree;
pub mod smart_label;
pub mod tree;

pub use session_analytics::SessionAnalytics;
pub use simple_tree::build_simple_tree;
pub use tree::TreeNode;
