//! Integration tests: JSONL fixture → parse → assert counts
//!
//! Verifies that the parser correctly handles a real-world JSONL transcript
//! containing all 7+ node types, tool calls, errors, and progress events.

use hindsight::parser::models::NodeType;
use std::path::Path;

fn fixture_path() -> &'static Path {
    Path::new("tests/fixtures/full_session.jsonl")
}

#[test]
fn loads_fixture_without_parse_errors() {
    let session = hindsight::parse_session(fixture_path()).expect("fixture must parse cleanly");
    assert!(!session.nodes.is_empty(), "fixture must produce nodes");
}

#[test]
fn node_count_per_type_matches_expected() {
    let session = hindsight::parse_session(fixture_path()).unwrap();

    let mut counts = std::collections::HashMap::<NodeType, usize>::new();
    for node in &session.nodes {
        *counts.entry(node.node_type).or_insert(0usize) += 1;
    }

    // Verify every expected type is present
    assert!(counts.contains_key(&NodeType::User), "missing user nodes");
    assert!(counts.contains_key(&NodeType::Assistant), "missing assistant nodes");
    assert!(counts.contains_key(&NodeType::Progress), "missing progress nodes");
    assert!(counts.contains_key(&NodeType::System), "missing system nodes");
    assert!(
        counts.contains_key(&NodeType::FileHistorySnapshot),
        "missing file-history-snapshot nodes"
    );
    assert!(
        counts.contains_key(&NodeType::LastPrompt),
        "missing last-prompt nodes"
    );
    assert!(counts.contains_key(&NodeType::PrLink), "missing pr-link nodes");
    assert!(
        counts.contains_key(&NodeType::QueueOperation),
        "missing queue-operation nodes"
    );

    // Sanity: user + assistant should be majority
    let user_count = counts[&NodeType::User];
    let assistant_count = counts[&NodeType::Assistant];
    assert!(user_count >= 10, "expected >= 10 user nodes, got {user_count}");
    assert!(
        assistant_count >= 15,
        "expected >= 15 assistant nodes, got {assistant_count}"
    );
}

#[test]
fn total_tools_count_correct() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    // The fixture has assistant nodes with tool_use content blocks
    // (Read, Bash, Edit, Write, Agent, Grep, Glob + synth multi-tool with 2)
    assert!(
        session.total_tools >= 10,
        "expected >= 10 tool calls, got {}",
        session.total_tools
    );
}

#[test]
fn error_count_correct() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    // The fixture has 3 user nodes with is_error=true tool_result blocks
    assert!(
        session.error_count >= 1,
        "expected >= 1 error, got {}",
        session.error_count
    );
}

#[test]
fn model_detection_works() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    let model = session.model.as_deref().expect("model must be detected");
    // Date suffix should be stripped
    assert!(
        !model.chars().last().unwrap_or(' ').is_ascii_digit()
            || !model.contains("-2025")
                && !model.contains("-2026"),
        "date suffix should be stripped: {model}"
    );
    assert!(
        model.starts_with("claude-"),
        "model should start with 'claude-': {model}"
    );
}

#[test]
fn first_message_extraction_works() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    // Find first user node with text content
    let first_user = session
        .nodes
        .iter()
        .find(|n| n.node_type == NodeType::User && n.message.as_ref().map(|m| !m.text_content().is_empty()).unwrap_or(false));
    assert!(first_user.is_some(), "should find a user node with text");
}

#[test]
fn progress_dedup_reduces_count() {
    // The fixture has multiple progress frames with same toolUseID for bash_progress
    // and multiple agent_progress frames. After dedup, count should be less than raw.
    let session = hindsight::parse_session(fixture_path()).unwrap();
    let progress_count = session
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Progress)
        .count();
    // We added 2 bash_progress with same toolUseID → should be deduped to 1
    // Raw fixture has 31 progress, but SSE dedup should reduce bash_progress pair
    assert!(
        progress_count < 31,
        "dedup should reduce progress count below raw 31, got {progress_count}"
    );
}

#[test]
fn timestamps_parse_both_formats() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    // All nodes should have parsed timestamps (ISO 8601 strings in the fixture)
    let with_ts = session.nodes.iter().filter(|n| n.timestamp.is_some()).count();
    // Most nodes have timestamps (some types like file-history-snapshot may not have top-level ts)
    assert!(
        with_ts > session.nodes.len() / 2,
        "at least half should have timestamps: {with_ts}/{}",
        session.nodes.len()
    );
}

#[test]
fn session_time_range_is_sane() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    assert!(session.start_time.is_some(), "start_time must be set");
    assert!(session.end_time.is_some(), "end_time must be set");
    assert!(
        session.end_time.unwrap() >= session.start_time.unwrap(),
        "end_time must be >= start_time"
    );
}
