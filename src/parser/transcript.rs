//! JSONL transcript parser implementation
//!
//! Parses Claude Code transcript files (JSONL format) into structured data.
//! Follows Rust best practices:
//! - Uses Result<T, E> for error handling
//! - Borrows where possible to avoid clones
//! - Iterator-based for memory efficiency

use super::models::{ContentBlock, ExecutionNode, MessageContent, Session, TokenUsage};
use crate::error::{HindsightError, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse a Claude Code JSONL transcript file into a Session
///
/// # Arguments
///
/// * `path` - Path to the .jsonl transcript file
///
/// # Returns
///
/// Returns a `Session` containing all parsed execution nodes with metadata.
///
/// # Errors
///
/// Returns `HindsightError` if:
/// - File cannot be read
/// - JSONL format is invalid
/// - JSON parsing fails
///
/// # Example
///
/// ```ignore
/// use hindsight::parser::parse_session;
/// use std::path::Path;
///
/// let session = parse_session(Path::new("session.jsonl"))?;
/// println!("Found {} tools", session.total_tools);
/// ```
/// Parse all subagent JSONL files associated with a session.
///
/// Subagent files live at `<parent>/<session_stem>/subagents/*.jsonl`.
/// Each subagent may use a different model (e.g., haiku spawned from a sonnet session).
pub fn parse_subagents(session_path: &Path) -> Vec<super::models::Session> {
    let session_id = session_path.file_stem().unwrap_or_default().to_string_lossy();
    let subagent_dir = session_path
        .parent()
        .map(|p| p.join(session_id.as_ref()).join("subagents"));

    let Some(dir) = subagent_dir else {
        return vec![];
    };
    if !dir.exists() {
        return vec![];
    }

    std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "jsonl"))
        .filter_map(|e| parse_session(&e.path()).ok())
        .collect()
}

pub fn parse_session(path: &Path) -> Result<Session> {
    let file = File::open(path)?;

    // Estimate initial capacity from file size to reduce reallocations
    let file_metadata = file.metadata()?;
    let file_size = file_metadata.len();
    let estimated_lines = (file_size / 500).max(100) as usize; // ~500 bytes per line

    let reader = BufReader::new(file);
    let mut raw_nodes = Vec::with_capacity(estimated_lines);

    // Parse JSONL line by line
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON line into ExecutionNode
        match serde_json::from_str::<ExecutionNode>(&line) {
            Ok(node) => {
                raw_nodes.push(node);
            }
            Err(e) => {
                return Err(HindsightError::JsonParse {
                    line: line_num + 1,
                    message: e.to_string(),
                });
            }
        }
    }

    // ── SSE message merging ────────────────────────────────────────────────────
    // Claude Code writes each content block of a response as a separate JSONL
    // line, all sharing the same message.id. Blocks are distinct and ordered:
    //   thinking → text → tool_use (one or more)
    // We accumulate all blocks for a given message.id into a single node, and
    // take the last token-usage record (which has cumulative counts).
    let mut merged: Vec<ExecutionNode> = Vec::with_capacity(raw_nodes.len());
    let mut current_id: Option<String> = None;
    let mut current_base: Option<ExecutionNode> = None;
    let mut current_content: Vec<ContentBlock> = Vec::new();
    let mut current_usage: Option<TokenUsage> = None;

    for node in raw_nodes {
        match extract_message_id(&node) {
            Some(id) if current_id.as_deref() == Some(id) => {
                // Same message — ACCUMULATE blocks (each line is a distinct block)
                // and keep the last token-usage (cumulative counts)
                let new_blocks = extract_blocks(&node);
                if !new_blocks.is_empty() {
                    current_content.extend(new_blocks);
                }
                if let Some(tu) = node.effective_token_usage() {
                    match current_usage.as_mut() {
                        Some(existing) => existing.merge_last(tu),
                        None => current_usage = Some(tu.clone()),
                    }
                }
            }
            Some(id) => {
                // New message.id — flush previous accumulator
                if let Some(base) = current_base.take() {
                    merged.push(finalize_sse(base, current_content, current_usage));
                }
                current_id = Some(id.to_string());
                current_content = extract_blocks(&node);
                current_usage = node.effective_token_usage().cloned();
                current_base = Some(node);
            }
            None => {
                // No message.id (tool results, progress, system) — flush and pass through
                if let Some(base) = current_base.take() {
                    merged.push(finalize_sse(base, current_content, current_usage));
                    current_id = None;
                    current_content = Vec::new();
                    current_usage = None;
                }
                merged.push(node);
            }
        }
    }
    // Flush final accumulator
    if let Some(base) = current_base.take() {
        merged.push(finalize_sse(base, current_content, current_usage));
    }

    let merged = merged;
    // ── end SSE deduplication ─────────────────────────────────────────────────

    // ── Progress node deduplication ───────────────────────────────────────────
    // Claude Code emits one progress frame per SSE tick for each running tool
    // (agent, bash, hook). All frames for the same invocation share the same
    // `toolUseID` but differ only by uuid/timestamp. Keep only the last frame
    // per toolUseID — it has the most complete output/elapsed time.
    let nodes = dedup_progress_nodes(merged);
    // ── end progress deduplication ────────────────────────────────────────────

    // Extract session ID from filename or first node
    let session_id = extract_session_id(path)?;

    // Get absolute path for display - try canonicalize, fallback to as-is
    let file_path = path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .or_else(|| path.to_str().map(String::from));

    Ok(Session::new(session_id, file_path, nodes))
}

/// Extract session ID from file path or content
fn extract_session_id(path: &Path) -> Result<String> {
    // Try to get session ID from filename
    if let Some(file_name) = path.file_stem() {
        if let Some(name) = file_name.to_str() {
            return Ok(name.to_string());
        }
    }

    Err(HindsightError::InvalidSession(
        "Could not extract session ID from path".to_string(),
    ))
}

// ── SSE helpers ───────────────────────────────────────────────────────────────

fn extract_message_id(node: &ExecutionNode) -> Option<&str> {
    node.message.as_ref()?.id.as_deref()
}

fn extract_blocks(node: &ExecutionNode) -> Vec<ContentBlock> {
    node.message
        .as_ref()
        .and_then(|m| m.content.as_ref())
        .map(|c| match c {
            MessageContent::Blocks(b) => b.clone(),
            MessageContent::Text(_) => vec![],
        })
        .unwrap_or_default()
}

fn finalize_sse(
    mut base: ExecutionNode,
    content: Vec<ContentBlock>,
    token_usage: Option<TokenUsage>,
) -> ExecutionNode {
    if let Some(ref mut msg) = base.message {
        if !content.is_empty() {
            msg.content = Some(MessageContent::Blocks(content));
        }
    }
    base.token_usage = token_usage;
    base
}

/// Extract the top-level `toolUseID` field from a node's flattened extra map.
fn extract_tool_use_id(node: &ExecutionNode) -> Option<String> {
    node.extra
        .as_ref()
        .and_then(|e| e.get("toolUseID"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Deduplicate progress nodes that share the same `toolUseID`.
///
/// For each running tool (agent, bash, hook), Claude Code writes one progress
/// node per SSE tick. All frames have the same prompt/content but a new uuid.
/// We keep only the LAST frame per toolUseID because it carries the most
/// complete data (highest elapsed time, fullest output, etc.).
fn dedup_progress_nodes(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    use std::collections::HashMap;

    // Record the index of the last occurrence of each toolUseID in a progress node.
    let mut last_idx: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.node_type == "progress" {
            if let Some(id) = extract_tool_use_id(node) {
                last_idx.insert(id, i);
            }
        }
    }

    // Retain non-progress nodes unchanged; for progress nodes keep only the last frame.
    nodes
        .into_iter()
        .enumerate()
        .filter(|(i, node)| {
            if node.node_type == "progress" {
                if let Some(id) = extract_tool_use_id(node) {
                    return last_idx.get(&id) == Some(i);
                }
            }
            true
        })
        .map(|(_, node)| node)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{Message, TokenUsage};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_empty_file() {
        let file = NamedTempFile::new().unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 0);
    }

    #[test]
    fn test_parse_user_message() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"type":"user","message":{{"content":"Hello"}}}}"#).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        assert_eq!(session.nodes[0].node_type, "user");
    }

    #[test]
    fn test_parse_tool_use() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"tool_use","tool_use":{{"name":"Read","input":{{"file_path":"test.txt"}}}}}}"#
        )
        .unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        assert!(session.nodes[0].tool_use.is_some());
        assert_eq!(session.total_tools, 1);
    }

    #[test]
    fn test_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{invalid json").unwrap();

        let result = parse_session(file.path());
        assert!(result.is_err());
    }

    // ── SSE deduplication tests ───────────────────────────────────────────────

    fn make_assistant_node(id: &str, text: &str, tokens_out: i64) -> ExecutionNode {
        ExecutionNode {
            uuid: Some(format!("uuid-{}", id)),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: "assistant".to_string(),
            message: Some(Message {
                id: Some(id.to_string()),
                role: Some("assistant".to_string()),
                model: None,
                content: Some(MessageContent::Blocks(vec![ContentBlock::Text {
                    text: text.to_string(),
                }])),
                usage: None,
                extra: HashMap::new(),
            }),
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: Some(TokenUsage {
                input_tokens: Some(100),
                output_tokens: Some(tokens_out),
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            }),
            extra: None,
        }
    }

    fn make_tool_node() -> ExecutionNode {
        ExecutionNode {
            uuid: Some("uuid-tool".to_string()),
            parent_uuid: None,
            timestamp: Some(2000),
            node_type: "tool_result".to_string(),
            message: None,
            tool_use: None,
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        }
    }

    #[test]
    fn test_sse_deduplication_single_message_id_produces_one_node() {
        let mut file = NamedTempFile::new().unwrap();

        // Two SSE frames with same message id
        let node1 = make_assistant_node("msg-abc", "partial", 10);
        let node2 = make_assistant_node("msg-abc", "full text here", 20);

        writeln!(file, "{}", serde_json::to_string(&node1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&node2).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();

        // Should be deduplicated to 1 node
        assert_eq!(session.nodes.len(), 1);
    }

    #[test]
    fn test_sse_deduplication_two_message_ids_produce_two_nodes() {
        let mut file = NamedTempFile::new().unwrap();

        let node1 = make_assistant_node("msg-aaa", "first message", 10);
        let node2 = make_assistant_node("msg-bbb", "second message", 20);

        writeln!(file, "{}", serde_json::to_string(&node1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&node2).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 2);
    }

    #[test]
    fn test_sse_deduplication_token_usage_takes_last_cumulative_value() {
        let mut file = NamedTempFile::new().unwrap();

        // SSE sends cumulative tokens — last frame has the final total
        let node1 = make_assistant_node("msg-xyz", "partial", 10);
        let node2 = make_assistant_node("msg-xyz", "complete response", 50);

        writeln!(file, "{}", serde_json::to_string(&node1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&node2).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);

        // Token usage should reflect the LAST SSE frame
        let usage = session.nodes[0].token_usage.as_ref().unwrap();
        assert_eq!(usage.output_tokens, Some(50));
    }

    #[test]
    fn test_sse_deduplication_non_assistant_nodes_pass_through_unchanged() {
        let mut file = NamedTempFile::new().unwrap();

        let tool = make_tool_node();
        writeln!(file, "{}", serde_json::to_string(&tool).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        assert_eq!(session.nodes[0].node_type, "tool_result");
    }

    // ── Progress deduplication tests ──────────────────────────────────────────

    fn make_progress_node(tool_use_id: &str, uuid: &str) -> ExecutionNode {
        let mut extra = HashMap::new();
        extra.insert("toolUseID".to_string(), serde_json::json!(tool_use_id));
        extra.insert(
            "data".to_string(),
            serde_json::json!({
                "type": "agent_progress",
                "agentId": "abc123",
                "prompt": "do something",
                "message": {},
                "normalizedMessages": []
            }),
        );
        ExecutionNode {
            uuid: Some(uuid.to_string()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: "progress".to_string(),
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

    #[test]
    fn test_progress_dedup_keeps_only_last_frame_per_tool_use_id() {
        let mut file = NamedTempFile::new().unwrap();

        // 3 frames for the same toolUseID
        let n1 = make_progress_node("tool-abc", "uuid-1");
        let n2 = make_progress_node("tool-abc", "uuid-2");
        let n3 = make_progress_node("tool-abc", "uuid-3");

        writeln!(file, "{}", serde_json::to_string(&n1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&n2).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&n3).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1, "3 frames should collapse to 1");
        assert_eq!(
            session.nodes[0].uuid,
            Some("uuid-3".to_string()),
            "last frame kept"
        );
    }

    #[test]
    fn test_progress_dedup_preserves_distinct_tool_use_ids() {
        let mut file = NamedTempFile::new().unwrap();

        // 2 frames for tool-A, 2 for tool-B
        let a1 = make_progress_node("tool-A", "uuid-a1");
        let a2 = make_progress_node("tool-A", "uuid-a2");
        let b1 = make_progress_node("tool-B", "uuid-b1");
        let b2 = make_progress_node("tool-B", "uuid-b2");

        writeln!(file, "{}", serde_json::to_string(&a1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&a2).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&b1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&b2).unwrap()).unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 2, "two distinct tool IDs → 2 nodes");
        assert_eq!(session.nodes[0].uuid, Some("uuid-a2".to_string()));
        assert_eq!(session.nodes[1].uuid, Some("uuid-b2".to_string()));
    }
}
