//! API response DTOs for web dashboard
//!
//! These types are optimized for JSON serialization and frontend consumption.

use serde::{Serialize, Deserialize};
use crate::analyzer::TreeNode;

/// Serializable node representation for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponse {
    pub uuid: Option<String>,
    pub node_type: String,
    pub label: String,
    pub color: String,        // Semantic: "cyan", "green", etc.
    pub summary: String,
    pub depth: usize,
    pub has_error: bool,
    pub timestamp: Option<i64>,
    pub children: Vec<NodeResponse>,

    #[serde(flatten)]
    pub data: serde_json::Value,  // Original node data
}

/// Tree response with statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeResponse {
    pub roots: Vec<NodeResponse>,
    pub total_nodes: usize,
    pub max_depth: usize,
}

/// Session statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatsResponse {
    pub session_id: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub total_nodes: usize,
    pub total_tools: usize,
    pub total_tokens: i64,
    pub estimated_cost: f64,
    pub error_count: usize,
}

impl NodeResponse {
    /// Convert TreeNode to API response
    pub fn from_tree_node(node: &TreeNode) -> Self {
        let (label, color) = crate::analyzer::smart_label::get_node_label(node);

        Self {
            uuid: node.node.uuid.clone(),
            node_type: node.node.node_type.clone(),
            label,
            color: color.to_string(),
            summary: String::new(), // TODO: Add summary logic
            depth: node.depth,
            has_error: node.node
                .tool_result
                .as_ref()
                .and_then(|r| r.is_error)
                .unwrap_or(false),
            timestamp: node.node.timestamp,
            children: node.children.iter().map(Self::from_tree_node).collect(),
            data: serde_json::to_value(&node.node).unwrap_or(serde_json::Value::Null),
        }
    }
}
