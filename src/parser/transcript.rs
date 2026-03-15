//! JSONL transcript parser implementation
//!
//! Parses Claude Code transcript files (JSONL format) into structured data.
//! Follows Rust best practices:
//! - Uses Result<T, E> for error handling
//! - Borrows where possible to avoid clones
//! - Iterator-based for memory efficiency

use super::models::{ContentBlock, ExecutionNode, MessageContent, NodeType, Session, TokenUsage};
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
    let nodes = dedup_progress_by_tool_use_id(merged);
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

/// Deduplicate progress nodes by `toolUseID`, keeping the LAST frame.
///
/// For each running tool (agent, bash, hook), Claude Code writes one progress
/// node per SSE tick. All frames share the same `toolUseID` but differ by uuid.
/// We keep the LAST frame because it carries the most complete data (highest
/// elapsed time, fullest output, final exit code).
///
/// **Why LAST?** SSE progress frames are cumulative — each tick overwrites
/// the previous output. The final frame has the complete picture.
///
/// See also: `dedup_agent_progress_by_agent_id` in `simple_tree.rs` which
/// does the opposite (keeps FIRST) for a different reason.
fn dedup_progress_by_tool_use_id(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    use std::collections::HashMap;

    // Record the index of the last occurrence of each toolUseID in a progress node.
    let mut last_idx: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.node_type == NodeType::Progress {
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
            if node.node_type == NodeType::Progress {
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
    use crate::parser::models::{Message, NodeType, TokenUsage};
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
        assert_eq!(session.nodes[0].node_type, NodeType::User);
    }

    #[test]
    fn test_parse_tool_use() {
        // Real Claude Code format: tool calls are ContentBlock::ToolUse inside
        // an assistant message, not a top-level tool_use field.
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","id":"msg-1","content":[{{"type":"tool_use","id":"tu-1","name":"Read","input":{{"file_path":"test.txt"}}}}]}}}}"#
        )
        .unwrap();

        let session = parse_session(file.path()).unwrap();
        assert_eq!(session.nodes.len(), 1);
        // Tool use is in message content blocks
        let blocks = session.nodes[0].message.as_ref().unwrap().content_blocks();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0], super::super::models::ContentBlock::ToolUse { ref name, .. } if name == "Read"));
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
            node_type: NodeType::Assistant,
            is_sidechain: None,
            session_id: None,
            cwd: None,
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
            node_type: NodeType::Unknown,
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
        assert_eq!(session.nodes[0].node_type, NodeType::Unknown);
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

    // ── Fixture-based tests (real Claude Code JSONL shapes) ───────────────────
    //
    // Helper: parse the (possibly multi-line) JSON string and write it as a
    // single compact line. JSONL requires one JSON object per line — if the
    // raw fixture string contains newlines the parser would reject it.
    fn write_jsonl_fixture(file: &mut impl std::io::Write, json: &str) {
        let value: serde_json::Value = serde_json::from_str(json)
            .expect("fixture JSON must be valid");
        writeln!(file, "{}", value).unwrap();
    }
    //
    // These fixtures match the exact JSON structure emitted by Claude Code in
    // production. They verify that every field that matters is deserialized
    // correctly — especially parentUuid (camelCase) which drives tree building.

    /// Real `user` node shape including parentUuid, sessionId, isSidechain, cwd.
    #[test]
    fn fixture_user_node_deserializes_parent_uuid() {
        let json = r#"{
            "parentUuid": "79b0d470-84e4-42ab-8c38-bcff7a0aa24a",
            "isSidechain": false,
            "userType": "external",
            "cwd": "/home/user/project",
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "version": "2.1.71",
            "type": "user",
            "message": {
                "role": "user",
                "content": "Hello world"
            },
            "uuid": "c9f738fe-c3ee-4a41-a3da-a70ea43149f5",
            "timestamp": "2026-03-09T20:36:13.828Z",
            "permissionMode": "default"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        assert_eq!(session.nodes.len(), 1);
        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::User);
        assert_eq!(node.uuid.as_deref(), Some("c9f738fe-c3ee-4a41-a3da-a70ea43149f5"));
        // THE critical field — tree hierarchy depends on this
        assert_eq!(
            node.parent_uuid.as_deref(),
            Some("79b0d470-84e4-42ab-8c38-bcff7a0aa24a"),
            "parentUuid (camelCase) must deserialize into parent_uuid"
        );
        assert_eq!(node.is_sidechain, Some(false));
        assert_eq!(node.session_id.as_deref(), Some("a5134111-4445-460d-9848-3652e0364cc3"));
        assert_eq!(node.cwd.as_deref(), Some("/home/user/project"));
    }

    /// Real `assistant` node with thinking + tool_use content blocks and token usage.
    #[test]
    fn fixture_assistant_node_with_thinking_and_tool_use() {
        let json = r#"{
            "parentUuid": "c9f738fe-c3ee-4a41-a3da-a70ea43149f5",
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "version": "2.1.71",
            "message": {
                "model": "claude-opus-4-6-20260101",
                "id": "msg_01E7qN343ih6AF31ZohVDNC4",
                "type": "message",
                "role": "assistant",
                "content": [
                    {
                        "type": "thinking",
                        "thinking": "Let me plan this carefully.",
                        "signature": "sig_abc123"
                    },
                    {
                        "type": "tool_use",
                        "id": "toolu_01MXE8tThu2BW3FvknemYN26",
                        "name": "Bash",
                        "input": { "command": "ls -la" }
                    }
                ],
                "stop_reason": "tool_use",
                "usage": {
                    "input_tokens": 1500,
                    "output_tokens": 80,
                    "cache_creation_input_tokens": 0,
                    "cache_read_input_tokens": 1200
                }
            },
            "requestId": "req_abc",
            "type": "assistant",
            "uuid": "87eefa21-2674-4cd8-a892-7dd6c18a0f85",
            "timestamp": "2026-03-09T20:36:22.317Z"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::Assistant);
        assert_eq!(
            node.parent_uuid.as_deref(),
            Some("c9f738fe-c3ee-4a41-a3da-a70ea43149f5")
        );

        let msg = node.message.as_ref().unwrap();
        assert_eq!(msg.model_short(), Some("claude-opus-4-6"));
        assert_eq!(msg.id.as_deref(), Some("msg_01E7qN343ih6AF31ZohVDNC4"));

        let blocks = msg.content_blocks();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0], crate::parser::models::ContentBlock::Thinking { .. }));
        assert!(matches!(blocks[1], crate::parser::models::ContentBlock::ToolUse { ref name, .. } if name == "Bash"));

        // Token usage from message.usage
        let usage = node.effective_token_usage().unwrap();
        assert_eq!(usage.input_tokens, Some(1500));
        assert_eq!(usage.output_tokens, Some(80));
        assert_eq!(usage.cache_read_input_tokens, Some(1200));

        // total_tools counts ContentBlock::ToolUse
        assert_eq!(session.total_tools, 1);
    }

    /// Real `user` node containing a `tool_result` content block (tool response).
    #[test]
    fn fixture_user_node_with_tool_result_block() {
        let json = r#"{
            "parentUuid": "87eefa21-2674-4cd8-a892-7dd6c18a0f85",
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_01MXE8tThu2BW3FvknemYN26",
                        "content": [{ "type": "text", "text": "file1.rs\nfile2.rs" }]
                    }
                ]
            },
            "uuid": "f1b2c3d4-0000-0000-0000-000000000001",
            "timestamp": "2026-03-09T20:36:23.000Z"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::User);
        assert_eq!(
            node.parent_uuid.as_deref(),
            Some("87eefa21-2674-4cd8-a892-7dd6c18a0f85")
        );

        let blocks = node.message.as_ref().unwrap().content_blocks();
        assert_eq!(blocks.len(), 1);
        assert!(
            matches!(blocks[0], crate::parser::models::ContentBlock::ToolResult { ref tool_use_id, .. }
                if tool_use_id == "toolu_01MXE8tThu2BW3FvknemYN26")
        );
    }

    /// Real `progress` node with `bash_progress` data — verifies data reaches `extra`.
    #[test]
    fn fixture_progress_bash_progress_data_accessible_via_extra() {
        let json = r#"{
            "parentUuid": "59880d07-e97a-462a-b93e-af460e5f8608",
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "type": "progress",
            "data": {
                "type": "bash_progress",
                "output": "",
                "fullOutput": "hello",
                "elapsedTimeSeconds": 3,
                "totalLines": 0,
                "totalBytes": 0,
                "taskId": "bso2475ff",
                "timeoutMs": 120000
            },
            "toolUseID": "bash-progress-0",
            "parentToolUseID": "toolu_01MXE8tThu2BW3FvknemYN26",
            "uuid": "c5efb084-fd8d-46c2-9961-7d32017013fb",
            "timestamp": "2026-03-09T20:43:55.501Z"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::Progress);
        assert_eq!(node.parent_uuid.as_deref(), Some("59880d07-e97a-462a-b93e-af460e5f8608"));

        // Progress data lives in extra["data"]
        let data = node.extra.as_ref()
            .and_then(|e| e.get("data"))
            .expect("progress data must be in extra[\"data\"]");
        assert_eq!(data.get("type").and_then(|t| t.as_str()), Some("bash_progress"));
        assert_eq!(data.get("elapsedTimeSeconds").and_then(|v| v.as_f64()), Some(3.0));
        assert_eq!(data.get("fullOutput").and_then(|v| v.as_str()), Some("hello"));
    }

    /// Real `progress` node with `agent_progress` data — verifies agent ID extraction.
    #[test]
    fn fixture_progress_agent_progress_data_accessible_via_extra() {
        let json = r#"{
            "parentUuid": "3df6921d-08eb-4ebd-b938-9ef91842a22a",
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "type": "progress",
            "data": {
                "type": "agent_progress",
                "agentId": "agent-abc123",
                "prompt": "Explore the codebase.",
                "message": {}
            },
            "toolUseID": "toolu_agent_01",
            "parentToolUseID": "toolu_agent_01",
            "uuid": "1e3d2e80-2fed-4876-a3eb-6d049630107b",
            "timestamp": "2026-03-09T20:36:22.317Z"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::Progress);

        let data = node.extra.as_ref()
            .and_then(|e| e.get("data"))
            .expect("agent_progress data must be in extra[\"data\"]");
        assert_eq!(data.get("type").and_then(|t| t.as_str()), Some("agent_progress"));
        assert_eq!(data.get("agentId").and_then(|v| v.as_str()), Some("agent-abc123"));
    }

    /// Real `progress` node with `hook_progress` data.
    #[test]
    fn fixture_progress_hook_progress_data_accessible_via_extra() {
        let json = r#"{
            "parentUuid": null,
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "type": "progress",
            "data": {
                "type": "hook_progress",
                "hookEvent": "SessionStart",
                "hookName": "SessionStart:startup",
                "command": "/usr/local/bin/claude-hindsight hook session-start"
            },
            "parentToolUseID": "4a0f7372-12b8-41fe-9ba0-adb589b649ac",
            "toolUseID": "4a0f7372-12b8-41fe-9ba0-adb589b649ac",
            "timestamp": "2026-03-09T20:32:19.071Z",
            "uuid": "79b0d470-84e4-42ab-8c38-bcff7a0aa24a"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::Progress);
        // parentUuid is null → no parent
        assert_eq!(node.parent_uuid, None);

        let data = node.extra.as_ref()
            .and_then(|e| e.get("data"))
            .expect("hook_progress data must be in extra[\"data\"]");
        assert_eq!(data.get("type").and_then(|t| t.as_str()), Some("hook_progress"));
        assert_eq!(data.get("hookEvent").and_then(|v| v.as_str()), Some("SessionStart"));
    }

    /// Real `system` node — stop_hook_summary subtype.
    #[test]
    fn fixture_system_stop_hook_summary_parses() {
        let json = r#"{
            "parentUuid": "b566be18-5b56-472c-936b-bf8856c055b3",
            "isSidechain": false,
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3",
            "slug": "bubbly-zooming-quokka",
            "type": "system",
            "subtype": "stop_hook_summary",
            "hookCount": 1,
            "hookInfos": [{ "command": "/usr/local/bin/claude-hindsight hook stop" }],
            "hookErrors": [],
            "preventedContinuation": false,
            "stopReason": "",
            "hasOutput": false,
            "level": "suggestion",
            "timestamp": "2026-03-09T20:44:20.015Z",
            "uuid": "185d0e88-785f-413d-a825-ecbe6d44ce7e",
            "toolUseID": "70eef7b4-fb90-4e73-a5b9-578eb55924f1"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::System);
        assert_eq!(
            node.parent_uuid.as_deref(),
            Some("b566be18-5b56-472c-936b-bf8856c055b3")
        );

        let subtype = node.extra.as_ref()
            .and_then(|e| e.get("subtype"))
            .and_then(|v| v.as_str());
        assert_eq!(subtype, Some("stop_hook_summary"));
    }

    /// Real `file-history-snapshot` node.
    #[test]
    fn fixture_file_history_snapshot_parses() {
        let json = r#"{
            "type": "file-history-snapshot",
            "messageId": "c9f738fe-c3ee-4a41-a3da-a70ea43149f5",
            "snapshot": {
                "messageId": "c9f738fe-c3ee-4a41-a3da-a70ea43149f5",
                "trackedFileBackups": {},
                "timestamp": "2026-03-09T20:36:13.828Z"
            },
            "isSnapshotUpdate": false
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::FileHistorySnapshot);
        // These nodes have no uuid/parentUuid — they don't participate in the tree
        assert_eq!(node.uuid, None);
        assert_eq!(node.parent_uuid, None);

        // snapshot data is accessible via extra
        assert!(node.extra.as_ref().and_then(|e| e.get("snapshot")).is_some());
    }

    /// Real `last-prompt` node — stores the last user message text.
    #[test]
    fn fixture_last_prompt_node_parses() {
        let json = r#"{
            "type": "last-prompt",
            "lastPrompt": "can you push it",
            "sessionId": "a5134111-4445-460d-9848-3652e0364cc3"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::LastPrompt);
        let prompt = node.extra.as_ref()
            .and_then(|e| e.get("lastPrompt"))
            .and_then(|v| v.as_str());
        assert_eq!(prompt, Some("can you push it"));
    }

    /// Real `pr-link` node — created when Claude Code opens a PR.
    #[test]
    fn fixture_pr_link_node_parses() {
        let json = r#"{
            "type": "pr-link",
            "sessionId": "48317b72-a9e4-4c5d-87a4-9f8a6f912e27",
            "prNumber": 1,
            "prUrl": "https://github.com/example/repo/pull/1",
            "prRepository": "example/repo",
            "timestamp": "2026-03-02T01:20:49.326Z"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::PrLink);
        let pr_url = node.extra.as_ref()
            .and_then(|e| e.get("prUrl"))
            .and_then(|v| v.as_str());
        assert_eq!(pr_url, Some("https://github.com/example/repo/pull/1"));
    }

    /// Real `queue-operation` node.
    #[test]
    fn fixture_queue_operation_node_parses() {
        let json = r#"{
            "type": "queue-operation",
            "operation": "enqueue",
            "timestamp": "2026-03-02T01:02:04.245Z",
            "sessionId": "48317b72-a9e4-4c5d-87a4-9f8a6f912e27",
            "content": "i see it http://localhost:3000"
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.node_type, NodeType::QueueOperation);
        let op = node.extra.as_ref()
            .and_then(|e| e.get("operation"))
            .and_then(|v| v.as_str());
        assert_eq!(op, Some("enqueue"));
    }

    /// Multi-node fixture: verifies parentUuid links are correct across a full
    /// user → assistant(tool_use) → user(tool_result) chain.
    #[test]
    fn fixture_parent_uuid_links_across_full_tool_chain() {
        let json = concat!(
            r#"{"type":"user","uuid":"node-1","parentUuid":null,"sessionId":"s1","message":{"role":"user","content":"do something"},"timestamp":1000}"#, "\n",
            r#"{"type":"assistant","uuid":"node-2","parentUuid":"node-1","sessionId":"s1","message":{"role":"assistant","id":"msg-1","content":[{"type":"tool_use","id":"toolu-1","name":"Bash","input":{"command":"ls"}}]},"timestamp":2000}"#, "\n",
            r#"{"type":"user","uuid":"node-3","parentUuid":"node-2","sessionId":"s1","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu-1","content":[{"type":"text","text":"ok"}]}]},"timestamp":3000}"#, "\n"
        );

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{json}").unwrap();
        let session = parse_session(file.path()).unwrap();

        assert_eq!(session.nodes.len(), 3);

        // Verify parent links
        assert_eq!(session.nodes[0].parent_uuid, None);
        assert_eq!(session.nodes[1].parent_uuid.as_deref(), Some("node-1"));
        assert_eq!(session.nodes[2].parent_uuid.as_deref(), Some("node-2"));

        // Verify tool_use is counted from content blocks
        assert_eq!(session.total_tools, 1);

        // Verify the tool result block is recognized
        let result_blocks = session.nodes[2].message.as_ref().unwrap().content_blocks();
        assert!(matches!(
            result_blocks[0],
            crate::parser::models::ContentBlock::ToolResult { ref tool_use_id, .. }
                if tool_use_id == "toolu-1"
        ));
    }

    /// Sidechain node (isSidechain: true) parses correctly.
    #[test]
    fn fixture_sidechain_node_parses_is_sidechain_field() {
        let json = r#"{
            "type": "user",
            "uuid": "side-node-1",
            "parentUuid": "main-node-1",
            "isSidechain": true,
            "sessionId": "s1",
            "message": { "role": "user", "content": "subagent prompt" },
            "timestamp": 1000
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        let node = &session.nodes[0];
        assert_eq!(node.is_sidechain, Some(true));
        assert_eq!(node.parent_uuid.as_deref(), Some("main-node-1"));
    }

    /// Agent tool call in assistant message — verifies Agent invocations are
    /// counted in total_tools just like Bash, Read, etc.
    #[test]
    fn fixture_agent_tool_call_counted_in_total_tools() {
        let json = r#"{
            "type": "assistant",
            "uuid": "node-agent",
            "parentUuid": "node-user",
            "sessionId": "s1",
            "message": {
                "role": "assistant",
                "id": "msg-agent-1",
                "content": [
                    {
                        "type": "tool_use",
                        "id": "toolu_agent_01",
                        "name": "Agent",
                        "input": {
                            "prompt": "Explore the project structure thoroughly.",
                            "subagent_type": "general-purpose"
                        }
                    }
                ]
            },
            "timestamp": 2000
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write_jsonl_fixture(&mut file, json);
        let session = parse_session(file.path()).unwrap();

        assert_eq!(session.total_tools, 1, "Agent tool call must be counted");

        let blocks = session.nodes[0].message.as_ref().unwrap().content_blocks();
        assert!(
            matches!(blocks[0], crate::parser::models::ContentBlock::ToolUse { ref name, .. } if name == "Agent")
        );
    }
}
