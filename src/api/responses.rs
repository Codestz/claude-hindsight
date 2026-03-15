//! API response DTOs for web dashboard
//!
//! These types are optimized for JSON serialization and frontend consumption.

use crate::analyzer::TreeNode;
use crate::parser::extract::{self, TokenUsageSummary};
use crate::parser::models::NodeType;
use serde::{Deserialize, Serialize};

/// Serializable node representation for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponse {
    pub uuid: Option<String>,
    pub node_type: NodeType,
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

    /// Tool name for assistant tool-call nodes (e.g. "Read", "Bash").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// File paths referenced in tool inputs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_paths: Vec<String>,

    /// Token usage for this node (typically on assistant nodes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsageSummary>,

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
    pub prev_node_type: Option<NodeType>,
}

impl NodeResponseContext {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Build summary, tool_name, and file_paths for a node using typed extractors.
/// Returns (summary, tool_name, file_paths).
fn build_node_extras(
    node: &crate::parser::ExecutionNode,
) -> (String, Option<String>, Vec<String>) {
    match node.node_type {
        NodeType::User => {
            if let Some(up) = extract::extract_user(node) {
                if up.is_tool_result {
                    // Tool result — show correlation if possible
                    let id_preview = up
                        .tool_use_ids
                        .first()
                        .map(|id| id.chars().take(12).collect::<String>())
                        .unwrap_or_default();
                    return (format!("Result for {}", id_preview), None, vec![]);
                }
                let preview = truncate_chars(&up.text, 120);
                return (preview, None, vec![]);
            }
            (String::new(), None, vec![])
        }
        NodeType::Assistant => {
            if let Some(resp) = extract::extract_assistant(node) {
                let file_paths = extract::extract_file_paths(node);

                // Determine primary tool name (first tool call)
                let tool_name = resp.tool_calls.first().map(|tc| tc.name.clone());

                let summary = if resp.thinking.is_some() && !resp.tool_calls.is_empty() {
                    format!("Thinking + {} tool call{}", resp.tool_calls.len(),
                        if resp.tool_calls.len() > 1 { "s" } else { "" })
                } else if resp.thinking.is_some() {
                    "Thinking".to_string()
                } else if let Some(tc) = resp.tool_calls.first() {
                    // Tool-specific summary
                    match tc.name.as_str() {
                        "Read" => {
                            let f = tc.file_path.as_deref().unwrap_or("file");
                            format!("Read: {}", short_filename(f))
                        }
                        "Write" | "Edit" => {
                            let f = tc.file_path.as_deref().unwrap_or("file");
                            format!("{}: {}", tc.name, short_filename(f))
                        }
                        "Bash" => {
                            let cmd = tc.input.get("command").and_then(|c| c.as_str()).unwrap_or("");
                            format!("Bash: {}", truncate_chars(cmd, 60))
                        }
                        "Grep" => {
                            let pat = tc.input.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
                            format!("Grep: {}", truncate_chars(pat, 40))
                        }
                        "Glob" => {
                            let pat = tc.input.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
                            format!("Glob: {}", pat)
                        }
                        "Agent" => {
                            let prompt = tc.input.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
                            format!("Agent: {}", truncate_chars(prompt, 50))
                        }
                        other => other.to_string(),
                    }
                } else {
                    truncate_chars(&resp.text, 120)
                };

                return (summary, tool_name, file_paths);
            }
            (String::new(), None, vec![])
        }
        NodeType::Progress => {
            if let Some(info) = extract::extract_progress(node) {
                let summary = match info.progress_type {
                    extract::ProgressType::Bash => format!("Bash: {}", info.details),
                    extract::ProgressType::Hook => format!("Hook: {}", info.details),
                    extract::ProgressType::Agent => format!("Agent: {}", info.details),
                    extract::ProgressType::Other(ref t) => format!("Progress: {}", t),
                };
                return (summary, None, vec![]);
            }
            (String::new(), None, vec![])
        }
        NodeType::System => {
            if let Some(info) = extract::extract_system(node) {
                return (info.details, None, vec![]);
            }
            (String::new(), None, vec![])
        }
        NodeType::FileHistorySnapshot => {
            let count = node
                .extra
                .as_ref()
                .and_then(|e| e.get("snapshot"))
                .and_then(|s| s.get("trackedFileBackups"))
                .and_then(|t| t.as_object())
                .map(|o| o.len())
                .unwrap_or(0);
            (format!("Snapshot: {} files", count), None, vec![])
        }
        NodeType::QueueOperation => {
            let op = node
                .extra
                .as_ref()
                .and_then(|e| e.get("operation"))
                .and_then(|v| v.as_str())
                .unwrap_or("queued");
            (op.to_string(), None, vec![])
        }
        NodeType::LastPrompt => {
            let text = node
                .extra
                .as_ref()
                .and_then(|e| e.get("lastPrompt"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            (truncate_chars(text, 120), None, vec![])
        }
        NodeType::PrLink => {
            let pr = node
                .extra
                .as_ref()
                .and_then(|e| e.get("prNumber"))
                .and_then(|v| v.as_i64())
                .map(|n| format!("PR #{}", n))
                .unwrap_or_else(|| "PR".to_string());
            (pr, None, vec![])
        }
        _ => (String::new(), None, vec![]),
    }
}

/// Truncate a string to at most `max` characters, appending "..." if truncated.
fn truncate_chars(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    let trimmed = s.trim();
    if trimmed.chars().count() <= max {
        trimmed.to_string()
    } else {
        format!("{}...", trimmed.chars().take(max).collect::<String>())
    }
}

/// Get the filename from a path (last component).
fn short_filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
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
        let prompt_score = if node.node.node_type == NodeType::User {
            let is_first = !ctx.seen_first_user;
            let is_after_assistant = ctx
                .prev_node_type
                .map(|t| t == NodeType::Assistant)
                .unwrap_or(false);

            ctx.seen_first_user = true;

            let score =
                crate::analyzer::prompt_detect::prompt_score(&node.node, is_first, is_after_assistant);
            if score > 0 { Some(score) } else { None }
        } else {
            None
        };

        // Track this node type for the next sibling
        ctx.prev_node_type = Some(node.node.node_type);

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

        // Extract typed data for summary, tool_name, file_paths, token_usage
        let (summary, tool_name, file_paths) = build_node_extras(&node.node);
        let token_usage = extract::extract_token_usage(&node.node);

        Self {
            uuid: node.node.uuid.clone(),
            node_type: node.node.node_type,
            label,
            color: color.to_string(),
            summary,
            depth: node.depth,
            has_error: node.node.has_error(),
            timestamp: node.node.timestamp,
            children,
            prompt_score,
            tool_name,
            file_paths,
            token_usage,
            data: serde_json::to_value(&*node.node).unwrap_or(serde_json::Value::Null),
        }
    }
}
