//! Data models for Claude Code JSONL transcript parsing
//!
//! Follows Rust best practices:
//! - Borrowing over cloning (uses &str where possible)
//! - Derives for common traits (Debug, Clone, Serialize, Deserialize)
//! - Clear documentation for all types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Custom deserializer for timestamp that handles both string and number formats
mod timestamp_format {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TimestampFormat {
            Number(i64),
            String(String),
        }

        match Option::<TimestampFormat>::deserialize(deserializer)? {
            None => Ok(None),
            Some(TimestampFormat::Number(n)) => Ok(Some(n)),
            Some(TimestampFormat::String(s)) => {
                // Parse ISO 8601 string to milliseconds
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| Some(dt.timestamp_millis()))
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}

/// A typed content block matching the 4 real Anthropic content block types.
///
/// Uses internally-tagged serde representation to match the JSON `"type"` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

/// Message content — handles both legacy string and modern block array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// A single execution node in the Claude Code session tree
///
/// Represents any event in the transcript: user messages, assistant responses,
/// tool calls, thinking blocks, etc. Nodes are linked via uuid/parentUuid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    /// Unique identifier for this node
    pub uuid: Option<String>,

    /// Parent node UUID (for building hierarchy)
    pub parent_uuid: Option<String>,

    /// Timestamp in milliseconds (accepts both ISO 8601 string and number)
    #[serde(default, deserialize_with = "timestamp_format::deserialize")]
    pub timestamp: Option<i64>,

    /// Node type (user, assistant, tool_use, etc.)
    #[serde(rename = "type")]
    pub node_type: String,

    /// Message content (for user/assistant messages)
    pub message: Option<Message>,

    /// Tool use details (for tool_use type)
    pub tool_use: Option<ToolUse>,

    /// Tool result (for tool_result events)
    pub tool_result: Option<ToolResult>,

    /// Tool use result (raw tool output - can be string or object)
    #[serde(rename = "toolUseResult")]
    pub tool_use_result: Option<serde_json::Value>,

    /// Thinking content (for thinking blocks)
    pub thinking: Option<String>,

    /// Progress updates
    pub progress: Option<Progress>,

    /// Token usage statistics
    pub token_usage: Option<TokenUsage>,

    /// Additional metadata (optional to save memory when not present)
    #[serde(flatten)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl ExecutionNode {
    /// Returns token usage for this node.
    ///
    /// Claude Code stores usage inside `message.usage`, not at the top level.
    /// This helper checks both places so callers don't need to know the layout.
    pub fn effective_token_usage(&self) -> Option<&TokenUsage> {
        self.token_usage
            .as_ref()
            .or_else(|| self.message.as_ref().and_then(|m| m.usage.as_ref()))
    }
}

/// Message content (user or assistant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID — used for SSE stream deduplication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Role (user, assistant, system)
    pub role: Option<String>,

    /// Model string e.g. "claude-sonnet-4-5-20250929"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Content (string or typed block array)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,

    /// Token usage — populated on assistant messages from the API response
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    /// Additional message metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Message {
    /// Returns typed content blocks (empty slice for legacy string content).
    pub fn content_blocks(&self) -> &[ContentBlock] {
        match &self.content {
            Some(MessageContent::Blocks(b)) => b.as_slice(),
            _ => &[],
        }
    }

    /// Returns all plain text, handling both legacy strings and block arrays.
    pub fn text_content(&self) -> String {
        match &self.content {
            Some(MessageContent::Text(s)) => s.clone(),
            Some(MessageContent::Blocks(blocks)) => blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
            None => String::new(),
        }
    }

    /// Model name with date suffix stripped.
    /// "claude-sonnet-4-5-20250929" -> "claude-sonnet-4-5"
    pub fn model_short(&self) -> Option<&str> {
        self.model.as_deref().map(strip_model_date_suffix)
    }
}

/// Strip an 8-digit date suffix from a model name (regex-free).
fn strip_model_date_suffix(model: &str) -> &str {
    if model.len() > 9 {
        let bytes = model.as_bytes();
        for i in (0..model.len().saturating_sub(8)).rev() {
            if bytes[i] == b'-' {
                let suffix = &model[i + 1..];
                if suffix.len() == 8 && suffix.bytes().all(|b| b.is_ascii_digit()) {
                    return &model[..i];
                }
            }
        }
    }
    model
}

/// Tool use (tool call) details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUse {
    /// Tool name (e.g., "Read", "Write", "Bash")
    pub name: String,

    /// Tool input parameters (JSON)
    pub input: serde_json::Value,

    /// Unique tool use ID
    pub id: Option<String>,
}

/// File information from toolUseResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,

    pub content: Option<String>,

    #[serde(rename = "numLines")]
    pub num_lines: Option<i64>,
}

/// Tool result (tool output) details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool use ID this result corresponds to
    pub tool_use_id: Option<String>,

    /// Result content (may have line numbers - prefer file.content)
    pub content: Option<String>,

    /// File information (clean content without line numbers)
    pub file: Option<FileInfo>,

    /// Whether tool succeeded
    pub is_error: Option<bool>,

    /// Error message if failed
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: Option<i64>,
}

/// Progress update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Progress message
    pub message: Option<String>,

    /// Progress percentage (0-100)
    pub percentage: Option<f64>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: Option<i64>,

    /// Output tokens
    pub output_tokens: Option<i64>,

    /// Cache creation tokens
    pub cache_creation_input_tokens: Option<i64>,

    /// Cache read tokens
    pub cache_read_input_tokens: Option<i64>,
}

impl TokenUsage {
    /// Total effective input tokens (base + cache creation + cache reads).
    /// Matches LangSmith: all three are billed as input, just at different rates.
    pub fn total_input(&self) -> i64 {
        self.input_tokens.unwrap_or(0)
            + self.cache_creation_input_tokens.unwrap_or(0)
            + self.cache_read_input_tokens.unwrap_or(0)
    }

    pub fn total_output(&self) -> i64 {
        self.output_tokens.unwrap_or(0)
    }

    pub fn total(&self) -> i64 {
        self.total_input() + self.total_output()
    }

    /// Take the LAST value for each field (SSE cumulative — later = more complete).
    pub fn merge_last(&mut self, other: &TokenUsage) {
        if other.input_tokens.is_some() {
            self.input_tokens = other.input_tokens;
        }
        if other.output_tokens.is_some() {
            self.output_tokens = other.output_tokens;
        }
        if other.cache_creation_input_tokens.is_some() {
            self.cache_creation_input_tokens = other.cache_creation_input_tokens;
        }
        if other.cache_read_input_tokens.is_some() {
            self.cache_read_input_tokens = other.cache_read_input_tokens;
        }
    }
}

/// Tool use result from user nodes (file operations)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseResult {
    /// Operation type (create, update, delete)
    #[serde(rename = "type")]
    pub operation_type: Option<String>,

    /// File path affected
    pub file_path: Option<String>,

    /// File content
    pub content: Option<String>,

    /// Structured patch information
    pub structured_patch: Option<serde_json::Value>,
}

/// Progress data (nested in progress nodes)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressData {
    /// Progress subtype (bash_progress, hook_progress, waiting_for_task)
    #[serde(rename = "type")]
    pub progress_type: Option<String>,

    /// Elapsed time in seconds (for bash_progress)
    pub elapsed_time_seconds: Option<f64>,

    /// Full output (for bash_progress)
    pub full_output: Option<String>,

    /// Exit code (for bash_progress)
    pub exit_code: Option<i32>,

    /// Hook name (for hook_progress)
    pub hook_name: Option<String>,

    /// Status (for hook_progress)
    pub status: Option<String>,

    /// Task description (for waiting_for_task)
    pub task_description: Option<String>,

    /// Task ID (for waiting_for_task)
    pub task_id: Option<String>,
}

/// Complete session parsed from JSONL transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session identifier
    pub session_id: String,

    /// Full file path to the JSONL file
    pub file_path: Option<String>,

    /// All execution nodes (flat list)
    pub nodes: Vec<ExecutionNode>,

    /// Session start time
    pub start_time: Option<i64>,

    /// Session end time
    pub end_time: Option<i64>,

    /// Total tool calls
    pub total_tools: usize,

    /// Total tokens used (includes cache tokens)
    pub total_tokens: i64,

    /// Estimated cost in USD
    pub estimated_cost: f64,

    /// Number of errors
    pub error_count: usize,

    /// Detected model (date suffix stripped)
    pub model: Option<String>,
}

impl Session {
    /// Create a new session from parsed nodes
    pub fn new(session_id: String, file_path: Option<String>, nodes: Vec<ExecutionNode>) -> Self {
        let total_tools = nodes.iter().filter(|n| n.tool_use.is_some()).count();

        // Token total: include ALL token types (cache creation + cache reads)
        let total_tokens: i64 = nodes
            .iter()
            .filter_map(|n| n.effective_token_usage())
            .map(|t| t.total())
            .sum();

        let error_count = nodes
            .iter()
            .filter(|n| {
                let tool_result_error = n
                    .tool_result
                    .as_ref()
                    .and_then(|r| r.is_error)
                    .unwrap_or(false);

                let tool_use_result_error = n
                    .tool_use_result
                    .as_ref()
                    .and_then(|v| {
                        serde_json::from_value::<ToolResult>(v.clone())
                            .ok()
                            .and_then(|r| r.is_error)
                    })
                    .unwrap_or(false);

                tool_result_error || tool_use_result_error
            })
            .count();

        let start_time = nodes.iter().filter_map(|n| n.timestamp).min();
        let end_time = nodes.iter().filter_map(|n| n.timestamp).max();

        // Model detection: find first assistant message with a model field, strip date suffix
        let model: Option<String> = nodes
            .iter()
            .filter_map(|n| n.message.as_ref())
            .filter_map(|m| m.model_short())
            .next()
            .map(str::to_string);

        // Per-model cost estimation with cache tier pricing
        let estimated_cost = {
            let inp: i64 = nodes
                .iter()
                .filter_map(|n| n.effective_token_usage())
                .map(|t| t.input_tokens.unwrap_or(0))
                .sum();
            let cw: i64 = nodes
                .iter()
                .filter_map(|n| n.effective_token_usage())
                .map(|t| t.cache_creation_input_tokens.unwrap_or(0))
                .sum();
            let cr: i64 = nodes
                .iter()
                .filter_map(|n| n.effective_token_usage())
                .map(|t| t.cache_read_input_tokens.unwrap_or(0))
                .sum();
            let out: i64 = nodes
                .iter()
                .filter_map(|n| n.effective_token_usage())
                .map(|t| t.output_tokens.unwrap_or(0))
                .sum();
            estimate_cost(model.as_deref().unwrap_or(""), inp, cw, cr, out)
        };

        Session {
            session_id,
            file_path,
            nodes,
            start_time,
            end_time,
            total_tools,
            total_tokens,
            estimated_cost,
            error_count,
            model,
        }
    }
}

/// Model-aware cost estimator (USD). Pricing as of early 2026.
/// (input_$/M, cache_write_$/M, cache_read_$/M, output_$/M)
fn estimate_cost(model: &str, inp: i64, cw: i64, cr: i64, out: i64) -> f64 {
    let (i, w, r, o): (f64, f64, f64, f64) = if model.contains("opus") {
        (15.0, 18.75, 1.5, 75.0)
    } else if model.contains("haiku") {
        (0.8, 1.0, 0.08, 4.0)
    } else {
        (3.0, 3.75, 0.30, 15.0) // Sonnet default
    };
    let m = 1_000_000.0_f64;
    (inp as f64 / m * i) + (cw as f64 / m * w) + (cr as f64 / m * r) + (out as f64 / m * o)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ContentBlock deserialization ──────────────────────────────────────────

    #[test]
    fn test_content_block_text_roundtrip() {
        let json = r#"{"type":"text","text":"hello world"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::Text { ref text } if text == "hello world"));
        let back = serde_json::to_string(&block).unwrap();
        assert!(back.contains("hello world"));
    }

    #[test]
    fn test_content_block_thinking_roundtrip() {
        let json = r#"{"type":"thinking","thinking":"deep thoughts"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::Thinking { thinking, .. } if thinking == "deep thoughts"));
    }

    #[test]
    fn test_content_block_tool_use_roundtrip() {
        let json = r#"{"type":"tool_use","id":"tu_123","name":"Read","input":{"file_path":"test.rs"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::ToolUse { name, .. } if name == "Read"));
    }

    #[test]
    fn test_content_block_tool_result_roundtrip() {
        let json = r#"{"type":"tool_result","tool_use_id":"tu_123","content":"result text","is_error":false}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::ToolResult { tool_use_id, .. } if tool_use_id == "tu_123"));
    }

    #[test]
    fn test_content_block_unknown_falls_through_to_value() {
        let json = r#"{"type":"future_type","data":"something"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::Unknown(_)));
    }

    // ── MessageContent ────────────────────────────────────────────────────────

    #[test]
    fn test_message_content_legacy_string_deserializes() {
        let json = r#""hello""#;
        let mc: MessageContent = serde_json::from_str(json).unwrap();
        assert!(matches!(mc, MessageContent::Text(_)));
    }

    #[test]
    fn test_message_content_block_array_deserializes() {
        let json = r#"[{"type":"text","text":"hi"}]"#;
        let mc: MessageContent = serde_json::from_str(json).unwrap();
        assert!(matches!(mc, MessageContent::Blocks(_)));
    }

    // ── Message helpers ───────────────────────────────────────────────────────

    fn make_message_with_content(content: MessageContent) -> Message {
        Message {
            id: None,
            role: Some("assistant".to_string()),
            model: None,
            content: Some(content),
            usage: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_message_text_content_from_string() {
        let msg = make_message_with_content(MessageContent::Text("hello".to_string()));
        assert_eq!(msg.text_content(), "hello");
    }

    #[test]
    fn test_message_text_content_from_blocks() {
        let blocks = vec![
            ContentBlock::Text { text: "line one".to_string() },
            ContentBlock::Thinking { thinking: "hidden".to_string(), signature: None },
            ContentBlock::Text { text: "line two".to_string() },
        ];
        let msg = make_message_with_content(MessageContent::Blocks(blocks));
        let text = msg.text_content();
        assert!(text.contains("line one"));
        assert!(text.contains("line two"));
        assert!(!text.contains("hidden"));
    }

    #[test]
    fn test_message_content_blocks_empty_for_string() {
        let msg = make_message_with_content(MessageContent::Text("x".to_string()));
        assert!(msg.content_blocks().is_empty());
    }

    // ── strip_model_date_suffix ───────────────────────────────────────────────

    #[test]
    fn test_strip_date_suffix_removes_8_digit_suffix() {
        assert_eq!(strip_model_date_suffix("claude-sonnet-4-5-20250929"), "claude-sonnet-4-5");
        assert_eq!(strip_model_date_suffix("claude-opus-4-6-20260101"), "claude-opus-4-6");
        assert_eq!(strip_model_date_suffix("claude-haiku-4-5-20251001"), "claude-haiku-4-5");
    }

    #[test]
    fn test_strip_date_suffix_no_change_when_no_suffix() {
        assert_eq!(strip_model_date_suffix("claude-sonnet-4-5"), "claude-sonnet-4-5");
        assert_eq!(strip_model_date_suffix("claude"), "claude");
        assert_eq!(strip_model_date_suffix(""), "");
    }

    // ── TokenUsage ────────────────────────────────────────────────────────────

    #[test]
    fn test_token_usage_total_includes_cache_tokens() {
        let tu = TokenUsage {
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_creation_input_tokens: Some(200),
            cache_read_input_tokens: Some(300),
        };
        assert_eq!(tu.total_input(), 600);
        assert_eq!(tu.total_output(), 50);
        assert_eq!(tu.total(), 650);
    }

    #[test]
    fn test_token_usage_merge_last_replaces_non_none_fields() {
        let mut base = TokenUsage {
            input_tokens: Some(10),
            output_tokens: Some(20),
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        let other = TokenUsage {
            input_tokens: Some(100),
            output_tokens: Some(200),
            cache_creation_input_tokens: Some(50),
            cache_read_input_tokens: None,
        };
        base.merge_last(&other);
        assert_eq!(base.input_tokens, Some(100));
        assert_eq!(base.output_tokens, Some(200));
        assert_eq!(base.cache_creation_input_tokens, Some(50));
        assert_eq!(base.cache_read_input_tokens, None);
    }

    #[test]
    fn test_token_usage_merge_last_preserves_none_fields() {
        let mut base = TokenUsage {
            input_tokens: Some(10),
            output_tokens: Some(20),
            cache_creation_input_tokens: Some(5),
            cache_read_input_tokens: Some(3),
        };
        let other = TokenUsage {
            input_tokens: None,
            output_tokens: None,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        base.merge_last(&other);
        // All fields preserved from base since other has None
        assert_eq!(base.input_tokens, Some(10));
        assert_eq!(base.output_tokens, Some(20));
        assert_eq!(base.cache_creation_input_tokens, Some(5));
        assert_eq!(base.cache_read_input_tokens, Some(3));
    }
}
