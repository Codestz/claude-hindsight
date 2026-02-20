//! API response DTOs for web dashboard
//!
//! These types are optimized for JSON serialization and frontend consumption.

use crate::analyzer::TreeNode;
use serde::{Deserialize, Serialize};

/// Serializable node representation for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponse {
    pub uuid: Option<String>,
    pub node_type: String,
    pub label: String,
    pub color: String, // Semantic: "cyan", "green", etc.
    pub summary: String,
    pub depth: usize,
    pub has_error: bool,
    pub timestamp: Option<i64>,
    pub children: Vec<NodeResponse>,

    #[serde(flatten)]
    pub data: serde_json::Value, // Original node data
}

/// Tree response with statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeResponse {
    pub roots: Vec<NodeResponse>,
    pub total_nodes: usize,
    pub max_depth: usize,
}

impl NodeResponse {
    /// Convert TreeNode to API response
    pub fn from_tree_node(node: &TreeNode) -> Self {
        let (label, color) = crate::analyzer::smart_label::get_node_label(node, None);

        Self {
            uuid: node.node.uuid.clone(),
            node_type: node.node.node_type.clone(),
            label,
            color: color.to_string(),
            summary: String::new(),
            depth: node.depth,
            has_error: {
                // Explicit is_error flag on top-level tool_result
                let tr = node.node.tool_result.as_ref();
                let flag_error = tr.and_then(|r| r.is_error).unwrap_or(false);

                // Content string containing <tool_use_error> tag
                let tag_error = tr
                    .and_then(|r| r.content.as_deref())
                    .map(|c| c.contains("<tool_use_error>"))
                    .unwrap_or(false);

                // ToolResult blocks inside message.content with is_error or error tag
                let block_error = node
                    .node
                    .message
                    .as_ref()
                    .map(|m| {
                        m.content_blocks().iter().any(|b| match b {
                            crate::parser::models::ContentBlock::ToolResult {
                                content, is_error, ..
                            } => {
                                is_error.unwrap_or(false)
                                    || content
                                        .as_ref()
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.contains("<tool_use_error>"))
                                        .unwrap_or(false)
                            }
                            _ => false,
                        })
                    })
                    .unwrap_or(false);

                flag_error || tag_error || block_error
            },
            timestamp: node.node.timestamp,
            children: node.children.iter().map(Self::from_tree_node).collect(),
            data: serde_json::to_value(&*node.node).unwrap_or(serde_json::Value::Null),
        }
    }
}
