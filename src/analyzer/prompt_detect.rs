//! Prompt detection with confidence scoring
//!
//! Computes a 0–100 confidence score indicating whether a user message
//! is a meaningful prompt (instruction, request, question) vs. a tool
//! result passthrough or minimal acknowledgement.

use crate::parser::models::{ContentBlock, ExecutionNode, NodeType};

/// Returns true if the text is system noise injected by the Claude Code client
/// (slash commands, stdout output, caveats, interruptions). These are not real
/// user prompts and should be filtered everywhere.
pub fn is_local_command_text(text: &str) -> bool {
    text.contains("<command-name>")
        || text.contains("<command-message>")
        || text.contains("<local-command-stdout>")
        || text.contains("<local-command-caveat>")
        || text.contains("<command-args>")
        || text.contains("[Request interrupted by user")
}

/// Returns true if the text is just a greeting, trivial chatter, a bare
/// file path (e.g. image drop), or system noise — not a meaningful prompt.
pub fn is_trivial_message(text: &str) -> bool {
    // Normalize: lowercase, strip all trailing/leading punctuation and whitespace
    let normalized: String = text
        .to_lowercase()
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .trim_start_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    const TRIVIAL: &[&str] = &[
        // Greetings
        "hey", "hi", "hello", "yo", "sup", "hola",
        "hey there", "hi there", "hello there",
        "hey how are you", "hi how are you", "hello how are you", "how are you",
        "hey how are you doing", "how are you doing",
        "what's up", "whats up",
        "good morning", "good afternoon", "good evening",
        "gm", "morning",
        // Acknowledgements
        "thanks", "thank you", "thx", "ty",
        "ok", "okay", "k", "sure", "yes", "no", "yep", "nope", "yeah", "nah",
        "got it", "sounds good", "looks good", "lgtm", "nice", "great", "cool",
        "perfect", "awesome", "alright", "right",
        // Continuations
        "go ahead", "continue", "proceed", "go on", "keep going",
        "do it", "go for it", "ship it",
        "y", "n",
    ];

    if TRIVIAL.iter().any(|t| normalized == *t) {
        return true;
    }

    // Bare file/image path — user dropped a file with no instruction text
    let stripped = text.trim().trim_matches('"');
    if stripped.starts_with('/') || stripped.starts_with("~/") {
        let has_ext = stripped.rsplit('/').next().map(|f| f.contains('.')).unwrap_or(false);
        let is_dir = stripped.ends_with('/');
        if has_ext || is_dir {
            return true;
        }
    }

    // System noise patterns
    if normalized.starts_with("unknown skill:") {
        return true;
    }

    false
}

/// Compute the prompt confidence score for a user node.
///
/// Returns 0–100. Scores >= 40 qualify as a "prompt" for filtering.
///
/// `is_first_in_session` — true if this is the very first user message.
/// `is_first_after_assistant` — true if the previous node was an assistant node
///   (i.e. this starts a new instruction turn, not a follow-up user message).
pub fn prompt_score(
    node: &ExecutionNode,
    is_first_in_session: bool,
    is_first_after_assistant: bool,
) -> u8 {
    // Only user nodes can be prompts
    if node.node_type != NodeType::User {
        return 0;
    }

    let msg = match node.message.as_ref() {
        Some(m) => m,
        None => return 0,
    };

    // Penalty: tool-result-only — message contains ONLY ToolResult blocks (no text)
    let blocks = msg.content_blocks();
    if !blocks.is_empty() {
        let has_text = blocks.iter().any(|b| matches!(b, ContentBlock::Text { text } if !text.trim().is_empty()));
        let has_tool_result = blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolResult { .. }));
        if has_tool_result && !has_text {
            return 0;
        }
    }

    let text = msg.text_content();
    let text = text.trim();

    // Filter: local commands, command output, and caveats — not real prompts
    if is_local_command_text(text) {
        return 0;
    }

    // Filter: greetings, acknowledgements, trivial chatter
    if is_trivial_message(text) {
        return 0;
    }

    let mut score: i32 = 0;

    // Signal: Position — first in session
    if is_first_in_session {
        score += 35;
    }

    // Signal: Position — first after assistant (new instruction turn)
    if is_first_after_assistant && !is_first_in_session {
        score += 20;
    }

    // Signal: Text length (0–20 points, scaled)
    let char_count = text.chars().count();
    let length_score = if char_count < 10 {
        0
    } else if char_count < 50 {
        // Linear scale 0→10 between 10 and 50 chars
        ((char_count - 10) as f64 / 40.0 * 10.0) as i32
    } else if char_count < 200 {
        // Linear scale 10→20 between 50 and 200 chars
        10 + ((char_count - 50) as f64 / 150.0 * 10.0) as i32
    } else {
        20
    };
    score += length_score;

    // Signal: Imperative keywords (+15)
    let text_lower = text.to_lowercase();
    let imperative_keywords = [
        "add", "fix", "create", "implement", "update", "change", "remove", "build", "write",
        "refactor", "debug", "test", "deploy", "configure", "set up", "migrate",
    ];
    if imperative_keywords
        .iter()
        .any(|kw| text_lower.contains(kw))
    {
        score += 15;
    }

    // Signal: Question/request patterns (+10)
    let request_patterns = [
        "how", "what", "why", "can you", "could you", "please", "i want", "i need", "help me",
    ];
    if request_patterns
        .iter()
        .any(|p| text_lower.contains(p))
    {
        score += 10;
    }

    // Penalty: very short text (-20)
    if char_count < 10 {
        score -= 20;
    }

    score.clamp(0, 100) as u8
}

/// Threshold for a node to qualify as a "prompt" in filters.
pub const PROMPT_THRESHOLD: u8 = 40;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{Message, MessageContent};
    use std::collections::HashMap;

    fn user_node(text: &str) -> ExecutionNode {
        ExecutionNode {
            uuid: Some("u1".to_string()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: NodeType::User,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: Some(Message {
                id: None,
                role: Some("user".to_string()),
                model: None,
                content: Some(MessageContent::Text(text.to_string())),
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

    fn tool_result_only_node() -> ExecutionNode {
        ExecutionNode {
            uuid: Some("u2".to_string()),
            parent_uuid: None,
            timestamp: Some(2000),
            node_type: NodeType::User,
            is_sidechain: None,
            session_id: None,
            cwd: None,
            message: Some(Message {
                id: None,
                role: Some("user".to_string()),
                model: None,
                content: Some(MessageContent::Blocks(vec![ContentBlock::ToolResult {
                    tool_use_id: "tu_1".to_string(),
                    content: Some(serde_json::json!("file contents here")),
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

    #[test]
    fn first_message_with_imperative_scores_high() {
        let node = user_node("Please add a new authentication system to the app");
        let score = prompt_score(&node, true, false);
        assert!(score >= 60, "Expected >= 60, got {}", score);
    }

    #[test]
    fn short_acknowledgement_scores_zero() {
        let node = user_node("ok");
        let score = prompt_score(&node, false, false);
        assert_eq!(score, 0);
    }

    #[test]
    fn tool_result_only_scores_zero() {
        let node = tool_result_only_node();
        let score = prompt_score(&node, false, false);
        assert_eq!(score, 0);
    }

    #[test]
    fn follow_up_instruction_scores_moderate() {
        let node = user_node("Now please fix the login page to handle errors properly");
        let score = prompt_score(&node, false, true);
        assert!(score >= 40, "Expected >= 40, got {}", score);
    }

    #[test]
    fn assistant_node_scores_zero() {
        let mut node = user_node("test");
        node.node_type = NodeType::Assistant;
        assert_eq!(prompt_score(&node, true, false), 0);
    }

    #[test]
    fn yes_scores_zero() {
        let node = user_node("yes");
        let score = prompt_score(&node, false, false);
        assert_eq!(score, 0);
    }

    #[test]
    fn local_command_scores_zero() {
        let node = user_node("<command-name>/model</command-name>\n<command-message>model</command-message>\n<command-args></command-args>");
        assert_eq!(prompt_score(&node, true, false), 0);
    }

    #[test]
    fn local_command_stdout_scores_zero() {
        let node = user_node("<local-command-stdout>Set model to sonnet</local-command-stdout>");
        assert_eq!(prompt_score(&node, false, true), 0);
    }

    #[test]
    fn local_command_caveat_scores_zero() {
        let node = user_node("<local-command-caveat>Caveat: The messages below were generated by the user while running local commands.</local-command-caveat>");
        assert_eq!(prompt_score(&node, false, false), 0);
    }

    #[test]
    fn interrupted_request_scores_zero() {
        let node = user_node("[Request interrupted by user for tool use]");
        assert_eq!(prompt_score(&node, true, false), 0);
    }

    #[test]
    fn greeting_scores_zero() {
        for greeting in &["hey", "Hi", "Hello", "hey how are you", "Hey there!", "yo", "thanks", "ok"] {
            let node = user_node(greeting);
            let score = prompt_score(&node, true, false);
            assert_eq!(score, 0, "Expected 0 for {:?}, got {}", greeting, score);
        }
    }

    #[test]
    fn trivial_message_detection() {
        assert!(is_trivial_message("hey"));
        assert!(is_trivial_message("Hey!"));
        assert!(is_trivial_message("Hi there"));
        assert!(is_trivial_message("thanks"));
        assert!(is_trivial_message("ok"));
        assert!(is_trivial_message("looks good"));
        assert!(is_trivial_message("Hey how are you ?."));
        assert!(is_trivial_message("Hey how are you?."));
        assert!(is_trivial_message("continue"));
        assert!(is_trivial_message("go ahead"));
        assert!(!is_trivial_message("Add authentication to the app"));
        assert!(!is_trivial_message("hey can you fix the login bug"));
    }

    #[test]
    fn bare_file_path_is_trivial() {
        assert!(is_trivial_message("\"/Users/me/Desktop/Screenshot 2026-02-23.png\""));
        assert!(is_trivial_message("/Users/me/photo.jpg"));
        assert!(is_trivial_message("~/Downloads/file.pdf"));
        assert!(!is_trivial_message("Look at /Users/me/file.png and fix the layout"));
    }

    #[test]
    fn system_noise_is_trivial() {
        assert!(is_trivial_message("Unknown skill: observatory:optimize-workflow"));
    }
}
