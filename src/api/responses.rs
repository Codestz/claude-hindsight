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

    /// Prompt confidence score (0–100). Only present on user nodes.
    /// Scores >= 40 indicate a meaningful prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_score: Option<u8>,

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

/// Context for building NodeResponses with prompt scoring
#[derive(Default)]
pub struct NodeResponseContext {
    /// Track whether we've seen the first user node
    pub seen_first_user: bool,
    /// The node_type of the previously visited node (for is_first_after_assistant)
    pub prev_node_type: Option<String>,
}

impl NodeResponseContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NodeResponse {
    /// Convert TreeNode to API response (without prompt scoring, for backward compat)
    pub fn from_tree_node(node: &TreeNode) -> Self {
        let mut ctx = NodeResponseContext::new();
        Self::from_tree_node_with_context(node, &mut ctx)
    }

    /// Convert TreeNode to API response with prompt scoring context
    pub fn from_tree_node_with_context(node: &TreeNode, ctx: &mut NodeResponseContext) -> Self {
        let (label, color) = crate::analyzer::smart_label::get_node_label(node, None);

        // Compute prompt score for user nodes
        let prompt_score = if node.node.node_type == "user" {
            let is_first = !ctx.seen_first_user;
            let is_after_assistant = ctx
                .prev_node_type
                .as_deref()
                .map(|t| t == "assistant")
                .unwrap_or(false);

            ctx.seen_first_user = true;

            let score =
                crate::analyzer::prompt_detect::prompt_score(&node.node, is_first, is_after_assistant);
            if score > 0 { Some(score) } else { None }
        } else {
            None
        };

        // Track this node type for the next sibling
        ctx.prev_node_type = Some(node.node.node_type.clone());

        // Build children with a fresh sub-context (children share parent's context)
        let mut child_ctx = NodeResponseContext {
            seen_first_user: ctx.seen_first_user,
            prev_node_type: None,
        };
        let children: Vec<NodeResponse> = node
            .children
            .iter()
            .map(|child| {
                let resp = Self::from_tree_node_with_context(child, &mut child_ctx);
                // Propagate seen_first_user back up
                ctx.seen_first_user = child_ctx.seen_first_user;
                resp
            })
            .collect();

        Self {
            uuid: node.node.uuid.clone(),
            node_type: node.node.node_type.clone(),
            label,
            color: color.to_string(),
            summary: String::new(),
            depth: node.depth,
            has_error: {
                let tr = node.node.tool_result.as_ref();
                let flag_error = tr.and_then(|r| r.is_error).unwrap_or(false);

                let tag_error = tr
                    .and_then(|r| r.content.as_deref())
                    .map(|c| c.contains("<tool_use_error>"))
                    .unwrap_or(false);

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
            children,
            prompt_score,
            data: serde_json::to_value(&*node.node).unwrap_or(serde_json::Value::Null),
        }
    }
}
