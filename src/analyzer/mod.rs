//! Execution tree analysis
//!
//! Builds hierarchical tree structures from flat JSONL execution nodes.

pub mod renderer;
pub mod simple_tree;
pub mod smart_label;
pub mod smart_tree;
pub mod tree;

pub use renderer::{render_tree, render_flat, render_stats, RenderConfig};
pub use simple_tree::build_simple_tree;
pub use smart_tree::build_conversation_tree;
pub use tree::{ExecutionTree, TreeNode, TreeStats};
