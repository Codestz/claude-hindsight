//! Execution tree analysis
//!
//! Builds hierarchical tree structures from flat JSONL execution nodes.

pub mod renderer;
pub mod smart_tree;
pub mod tree;

pub use renderer::{render_tree, render_flat, render_stats, RenderConfig};
pub use smart_tree::build_conversation_tree;
pub use tree::{ExecutionTree, TreeNode, TreeStats};
