//! Execution tree builder
//!
//! Transforms flat JSONL nodes into hierarchical tree structure for visualization.

use crate::parser::ExecutionNode;
use serde::{Deserialize, Serialize};

/// A node in the execution tree with children
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// The execution node data
    pub node: ExecutionNode,

    /// Child nodes
    pub children: Vec<TreeNode>,

    /// Depth in the tree (0 = root)
    pub depth: usize,
}

impl TreeNode {
    /// Get the timestamp of this node in milliseconds
    pub fn timestamp(&self) -> Option<i64> {
        self.node.timestamp
    }
}

