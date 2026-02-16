//! Execution tree builder
//!
//! Transforms flat JSONL nodes into hierarchical tree structure for visualization.

use crate::parser::ExecutionNode;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// A node in the execution tree with children
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// The execution node data (wrapped in Rc to avoid expensive cloning)
    #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
    pub node: Rc<ExecutionNode>,

    /// Child nodes
    pub children: Vec<TreeNode>,

    /// Depth in the tree (0 = root)
    pub depth: usize,
}

// Custom serialization for Rc<ExecutionNode>
fn serialize_rc<S>(node: &Rc<ExecutionNode>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    node.as_ref().serialize(serializer)
}

// Custom deserialization for Rc<ExecutionNode>
fn deserialize_rc<'de, D>(deserializer: D) -> Result<Rc<ExecutionNode>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    ExecutionNode::deserialize(deserializer).map(Rc::new)
}

impl TreeNode {
    // TreeNode methods can be added here as needed
}

