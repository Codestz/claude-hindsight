//! Typed node extractors — reusable across summary, labeling, and frontend.
//!
//! Each function takes an `ExecutionNode` reference and returns a typed struct
//! with the relevant data extracted from the node's content blocks, extra map,
//! and legacy fields.

use super::models::{ContentBlock, ExecutionNode, NodeType};

// ── Extracted types ──────────────────────────────────────────────────────────

/// Extracted user prompt information.
#[derive(Debug, Clone)]
pub struct UserPrompt {
    /// Plain text content (empty if tool-result-only).
    pub text: String,
    /// True if the message contains only ToolResult blocks (no user text).
    pub is_tool_result: bool,
    /// Tool use IDs from ToolResult blocks (for correlation).
    pub tool_use_ids: Vec<String>,
}

/// A single tool call extracted from an assistant message.
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    /// Primary file path extracted from tool input (if any).
    pub file_path: Option<String>,
}

/// Extracted assistant response information.
#[derive(Debug, Clone)]
pub struct AssistantResponse {
    /// Plain text blocks concatenated.
    pub text: String,
    /// Thinking block content (if any).
    pub thinking: Option<String>,
    /// Tool calls in this message.
    pub tool_calls: Vec<ToolCallInfo>,
}

/// Progress event subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressType {
    Bash,
    Hook,
    Agent,
    Other(String),
}

/// Extracted progress information.
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub progress_type: ProgressType,
    /// Human-readable detail string (elapsed time, hook name, agent prompt, etc.)
    pub details: String,
}

/// System event subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemSubtype {
    CompactBoundary,
    TurnDuration,
    StopHookSummary,
    Other(String),
}

/// Extracted system node information.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub subtype: SystemSubtype,
    /// Human-readable detail string.
    pub details: String,
}

// ── Extraction functions ─────────────────────────────────────────────────────

/// Extract user prompt data from a user node.
pub fn extract_user(node: &ExecutionNode) -> Option<UserPrompt> {
    if node.node_type != NodeType::User {
        return None;
    }
    let msg = node.message.as_ref()?;
    let blocks = msg.content_blocks();

    let tool_use_ids: Vec<String> = blocks
        .iter()
        .filter_map(|b| {
            if let ContentBlock::ToolResult { tool_use_id, .. } = b {
                Some(tool_use_id.clone())
            } else {
                None
            }
        })
        .collect();

    let has_text = blocks
        .iter()
        .any(|b| matches!(b, ContentBlock::Text { text } if !text.trim().is_empty()));
    let has_tool_result = !tool_use_ids.is_empty();

    let is_tool_result = has_tool_result && !has_text;
    let text = msg.text_content();

    Some(UserPrompt {
        text,
        is_tool_result,
        tool_use_ids,
    })
}

/// Extract assistant response data from an assistant node.
pub fn extract_assistant(node: &ExecutionNode) -> Option<AssistantResponse> {
    if node.node_type != NodeType::Assistant {
        return None;
    }
    let msg = node.message.as_ref()?;
    let blocks = msg.content_blocks();

    let mut text_parts = Vec::new();
    let mut thinking = None;
    let mut tool_calls = Vec::new();

    for block in blocks {
        match block {
            ContentBlock::Text { text } => text_parts.push(text.clone()),
            ContentBlock::Thinking {
                thinking: thought, ..
            } => {
                thinking = Some(thought.clone());
            }
            ContentBlock::ToolUse { id, name, input } => {
                let file_path = extract_file_path_from_input(name, input);
                tool_calls.push(ToolCallInfo {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    file_path,
                });
            }
            _ => {}
        }
    }

    Some(AssistantResponse {
        text: text_parts.join("\n\n"),
        thinking,
        tool_calls,
    })
}

/// Extract all tool calls from a node (convenience for assistant nodes).
pub fn extract_tool_calls(node: &ExecutionNode) -> Vec<ToolCallInfo> {
    extract_assistant(node)
        .map(|a| a.tool_calls)
        .unwrap_or_default()
}

/// Extract progress information from a progress node.
pub fn extract_progress(node: &ExecutionNode) -> Option<ProgressInfo> {
    if node.node_type != NodeType::Progress {
        return None;
    }
    let data = node.extra.as_ref()?.get("data")?;
    let type_str = data.get("type")?.as_str()?;

    match type_str {
        "bash_progress" => {
            let elapsed = data
                .get("elapsedTimeSeconds")
                .and_then(|e| e.as_f64())
                .map(|s| format!("{:.1}s", s))
                .unwrap_or_else(|| "running".to_string());
            Some(ProgressInfo {
                progress_type: ProgressType::Bash,
                details: elapsed,
            })
        }
        "hook_progress" => {
            let hook_name = data
                .get("hookName")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string();
            Some(ProgressInfo {
                progress_type: ProgressType::Hook,
                details: hook_name,
            })
        }
        "agent_progress" => {
            let label = data
                .get("prompt")
                .and_then(|p| p.as_str())
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let cleaned = p.replace('\n', " ");
                    let trimmed = cleaned.trim().to_string();
                    if trimmed.chars().count() > 50 {
                        format!("{}…", trimmed.chars().take(50).collect::<String>())
                    } else {
                        trimmed
                    }
                })
                .unwrap_or_else(|| {
                    data.get("agentId")
                        .and_then(|a| a.as_str())
                        .unwrap_or("unknown")
                        .chars()
                        .take(8)
                        .collect()
                });
            Some(ProgressInfo {
                progress_type: ProgressType::Agent,
                details: label,
            })
        }
        other => Some(ProgressInfo {
            progress_type: ProgressType::Other(other.to_string()),
            details: other.to_string(),
        }),
    }
}

/// Extract system node information.
pub fn extract_system(node: &ExecutionNode) -> Option<SystemInfo> {
    if node.node_type != NodeType::System {
        return None;
    }
    let extra = node.extra.as_ref()?;
    let subtype_str = extra.get("subtype").and_then(|s| s.as_str());

    match subtype_str {
        Some("compact_boundary") => {
            let pre_tokens = extra
                .get("compactMetadata")
                .and_then(|m| m.get("preTokens"))
                .and_then(|v| v.as_i64());
            let detail = match pre_tokens {
                Some(t) if t >= 1_000_000 => {
                    format!("Compacted {:.1}M tokens", t as f64 / 1_000_000.0)
                }
                Some(t) if t >= 1_000 => {
                    format!("Compacted {:.1}K tokens", t as f64 / 1_000.0)
                }
                Some(t) => format!("Compacted {} tokens", t),
                None => "Compacted".to_string(),
            };
            Some(SystemInfo {
                subtype: SystemSubtype::CompactBoundary,
                details: detail,
            })
        }
        Some("turn_duration") => {
            let detail = extra
                .get("durationMs")
                .and_then(|d| d.as_i64())
                .map(|ms| format!("Turn {:.1}s", ms as f64 / 1000.0))
                .unwrap_or_else(|| "Turn duration".to_string());
            Some(SystemInfo {
                subtype: SystemSubtype::TurnDuration,
                details: detail,
            })
        }
        Some("stop_hook_summary") => Some(SystemInfo {
            subtype: SystemSubtype::StopHookSummary,
            details: "Stop hook".to_string(),
        }),
        Some(other) => Some(SystemInfo {
            subtype: SystemSubtype::Other(other.to_string()),
            details: other.to_string(),
        }),
        None => Some(SystemInfo {
            subtype: SystemSubtype::Other("unknown".to_string()),
            details: "event".to_string(),
        }),
    }
}

/// Extract file paths from a node (tool inputs and tool results).
pub fn extract_file_paths(node: &ExecutionNode) -> Vec<String> {
    let mut paths = Vec::new();

    // From tool calls in assistant messages
    if let Some(resp) = extract_assistant(node) {
        for tc in &resp.tool_calls {
            if let Some(ref p) = tc.file_path {
                paths.push(p.clone());
            }
        }
    }

    // From legacy top-level tool_use
    if let Some(ref tu) = node.tool_use {
        if let Some(p) = extract_file_path_from_input(&tu.name, &tu.input) {
            paths.push(p);
        }
    }

    paths
}

/// Extract file path from tool input JSON based on tool name.
fn extract_file_path_from_input(tool_name: &str, input: &serde_json::Value) -> Option<String> {
    match tool_name {
        "Read" | "Write" | "Edit" => input.get("file_path").and_then(|v| v.as_str()).map(normalize_path),
        "Glob" => input.get("path").and_then(|v| v.as_str()).map(normalize_path),
        "Grep" => input.get("path").and_then(|v| v.as_str()).map(normalize_path),
        _ => None,
    }
}

/// Normalize a file path: strip common prefixes and resolve relative paths.
fn normalize_path(path: &str) -> String {
    // Strip common absolute path prefixes to get relative paths
    let path = path.trim();
    path.to_string()
}

// ── Token usage summary ──────────────────────────────────────────────────────

/// Summarized token usage for API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenUsageSummary {
    pub input: i64,
    pub output: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_create: Option<i64>,
}

/// Extract token usage summary from a node.
pub fn extract_token_usage(node: &ExecutionNode) -> Option<TokenUsageSummary> {
    let tu = node.effective_token_usage()?;
    Some(TokenUsageSummary {
        input: tu.input_tokens.unwrap_or(0),
        output: tu.output_tokens.unwrap_or(0),
        cache_read: tu.cache_read_input_tokens,
        cache_create: tu.cache_creation_input_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{Message, MessageContent, TokenUsage};
    use std::collections::HashMap;

    fn make_user_text_node(text: &str) -> ExecutionNode {
        ExecutionNode {
            uuid: Some("u1".into()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: NodeType::User,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: Some(Message {
                id: None,
                role: Some("user".into()),
                model: None,
                content: Some(MessageContent::Text(text.into())),
                usage: None,
                extra: HashMap::new(),
            }),
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        }
    }

    fn make_user_tool_result_node(tool_use_id: &str) -> ExecutionNode {
        ExecutionNode {
            uuid: Some("u2".into()),
            parent_uuid: None,
            timestamp: Some(2000),
            node_type: NodeType::User,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: Some(Message {
                id: None,
                role: Some("user".into()),
                model: None,
                content: Some(MessageContent::Blocks(vec![ContentBlock::ToolResult {
                    tool_use_id: tool_use_id.into(),
                    content: Some(serde_json::json!("ok")),
                    is_error: None,
                }])),
                usage: None,
                extra: HashMap::new(),
            }),
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        }
    }

    fn make_assistant_node() -> ExecutionNode {
        ExecutionNode {
            uuid: Some("a1".into()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: NodeType::Assistant,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: Some(Message {
                id: Some("msg-1".into()),
                role: Some("assistant".into()),
                model: Some("claude-opus-4-6-20260101".into()),
                content: Some(MessageContent::Blocks(vec![
                    ContentBlock::Thinking {
                        thinking: "Let me think.".into(),
                        signature: None,
                    },
                    ContentBlock::Text {
                        text: "Here's my response.".into(),
                    },
                    ContentBlock::ToolUse {
                        id: "tu-1".into(),
                        name: "Read".into(),
                        input: serde_json::json!({"file_path": "src/main.rs"}),
                    },
                ])),
                usage: Some(TokenUsage {
                    input_tokens: Some(100),
                    output_tokens: Some(50),
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: Some(80),
                }),
                extra: HashMap::new(),
            }),
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        }
    }

    fn make_progress_node(progress_type: &str, data: serde_json::Value) -> ExecutionNode {
        let mut extra = HashMap::new();
        extra.insert("data".to_string(), data);
        ExecutionNode {
            uuid: Some("p1".into()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: NodeType::Progress,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: None,
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: Some(extra),
        }
    }

    // ── User extraction ──────────────────────────────────────────

    #[test]
    fn extract_user_text_message() {
        let node = make_user_text_node("Hello world");
        let prompt = extract_user(&node).unwrap();
        assert_eq!(prompt.text, "Hello world");
        assert!(!prompt.is_tool_result);
        assert!(prompt.tool_use_ids.is_empty());
    }

    #[test]
    fn extract_user_tool_result_only() {
        let node = make_user_tool_result_node("tu-123");
        let prompt = extract_user(&node).unwrap();
        assert!(prompt.is_tool_result);
        assert_eq!(prompt.tool_use_ids, vec!["tu-123"]);
    }

    #[test]
    fn extract_user_returns_none_for_assistant() {
        let node = make_assistant_node();
        assert!(extract_user(&node).is_none());
    }

    // ── Assistant extraction ─────────────────────────────────────

    #[test]
    fn extract_assistant_full_response() {
        let node = make_assistant_node();
        let resp = extract_assistant(&node).unwrap();
        assert!(resp.thinking.is_some());
        assert_eq!(resp.thinking.unwrap(), "Let me think.");
        assert_eq!(resp.text, "Here's my response.");
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "Read");
        assert_eq!(
            resp.tool_calls[0].file_path.as_deref(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn extract_tool_calls_from_assistant() {
        let node = make_assistant_node();
        let calls = extract_tool_calls(&node);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "Read");
    }

    // ── Progress extraction ──────────────────────────────────────

    #[test]
    fn extract_bash_progress() {
        let node = make_progress_node(
            "bash_progress",
            serde_json::json!({
                "type": "bash_progress",
                "elapsedTimeSeconds": 3.1,
            }),
        );
        let info = extract_progress(&node).unwrap();
        assert_eq!(info.progress_type, ProgressType::Bash);
        assert_eq!(info.details, "3.1s");
    }

    #[test]
    fn extract_hook_progress() {
        let node = make_progress_node(
            "hook_progress",
            serde_json::json!({
                "type": "hook_progress",
                "hookName": "SessionStart:startup",
            }),
        );
        let info = extract_progress(&node).unwrap();
        assert_eq!(info.progress_type, ProgressType::Hook);
        assert_eq!(info.details, "SessionStart:startup");
    }

    #[test]
    fn extract_agent_progress_with_prompt() {
        let node = make_progress_node(
            "agent_progress",
            serde_json::json!({
                "type": "agent_progress",
                "agentId": "abc123",
                "prompt": "Explore the codebase",
            }),
        );
        let info = extract_progress(&node).unwrap();
        assert_eq!(info.progress_type, ProgressType::Agent);
        assert_eq!(info.details, "Explore the codebase");
    }

    // ── File path extraction ─────────────────────────────────────

    #[test]
    fn extract_file_paths_from_read_tool() {
        let node = make_assistant_node();
        let paths = extract_file_paths(&node);
        assert_eq!(paths, vec!["src/main.rs"]);
    }

    // ── Token usage extraction ───────────────────────────────────

    #[test]
    fn extract_token_usage_from_assistant() {
        let node = make_assistant_node();
        let usage = extract_token_usage(&node).unwrap();
        assert_eq!(usage.input, 100);
        assert_eq!(usage.output, 50);
        assert_eq!(usage.cache_read, Some(80));
    }
}
