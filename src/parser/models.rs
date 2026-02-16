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

/// Message content (user or assistant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Content (can be string or array of content blocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,

    /// Role (user, assistant, system)
    pub role: Option<String>,

    /// Additional message metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Total tokens used
    pub total_tokens: i64,

    /// Estimated cost in USD
    pub estimated_cost: f64,

    /// Number of errors
    pub error_count: usize,
}

impl Session {
    /// Create a new session from parsed nodes
    pub fn new(session_id: String, file_path: Option<String>, nodes: Vec<ExecutionNode>) -> Self {
        let total_tools = nodes
            .iter()
            .filter(|n| n.tool_use.is_some())
            .count();
        
        let total_tokens: i64 = nodes
            .iter()
            .filter_map(|n| n.token_usage.as_ref())
            .map(|t| {
                t.input_tokens.unwrap_or(0) + t.output_tokens.unwrap_or(0)
            })
            .sum();
        
        let error_count = nodes
            .iter()
            .filter(|n| {
                // Check tool_result for errors
                let tool_result_error = n.tool_result
                    .as_ref()
                    .and_then(|r| r.is_error)
                    .unwrap_or(false);

                // Check tool_use_result for errors
                let tool_use_result_error = n.tool_use_result
                    .as_ref()
                    .and_then(|v| {
                        // Try to parse as ToolResult
                        serde_json::from_value::<ToolResult>(v.clone())
                            .ok()
                            .and_then(|r| r.is_error)
                    })
                    .unwrap_or(false);

                tool_result_error || tool_use_result_error
            })
            .count();
        
        let start_time = nodes
            .iter()
            .filter_map(|n| n.timestamp)
            .min();
        
        let end_time = nodes
            .iter()
            .filter_map(|n| n.timestamp)
            .max();
        
        // Estimate cost (rough approximation based on Sonnet 4.5 pricing)
        // $3 per million input tokens, $15 per million output tokens
        let input_tokens: i64 = nodes
            .iter()
            .filter_map(|n| n.token_usage.as_ref())
            .filter_map(|t| t.input_tokens)
            .sum();
        
        let output_tokens: i64 = nodes
            .iter()
            .filter_map(|n| n.token_usage.as_ref())
            .filter_map(|t| t.output_tokens)
            .sum();
        
        let estimated_cost = 
            (input_tokens as f64 / 1_000_000.0 * 3.0) +
            (output_tokens as f64 / 1_000_000.0 * 15.0);
        
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
        }
    }
}
