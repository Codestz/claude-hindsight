//! Snapshot tests — freeze NodeResponse JSON output so format changes are caught by CI.
//!
//! Run `cargo insta review` to accept initial snapshots or review changes.

use hindsight::api::responses::{NodeResponse, NodeResponseContext};
use hindsight::parser::models::NodeType;
use std::path::Path;

fn fixture_path() -> &'static Path {
    Path::new("tests/fixtures/full_session.jsonl")
}

fn build_all_responses() -> Vec<NodeResponse> {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    let tree_roots = hindsight::build_simple_tree(session.nodes);
    let mut ctx = NodeResponseContext::new();
    tree_roots
        .iter()
        .map(|root| NodeResponse::from_tree_node_with_context(root, &mut ctx))
        .collect()
}

fn collect_all(roots: &[NodeResponse]) -> Vec<&NodeResponse> {
    let mut out = Vec::new();
    fn collect<'a>(nodes: &'a [NodeResponse], out: &mut Vec<&'a NodeResponse>) {
        for n in nodes {
            out.push(n);
            collect(&n.children, out);
        }
    }
    collect(roots, &mut out);
    out
}

fn find_first_by_type<'a>(nodes: &[&'a NodeResponse], nt: NodeType) -> &'a NodeResponse {
    nodes
        .iter()
        .find(|n| n.node_type == nt)
        .unwrap_or_else(|| panic!("no node of type {:?} found", nt))
}

fn find_first_tool_call<'a>(nodes: &[&'a NodeResponse]) -> &'a NodeResponse {
    nodes
        .iter()
        .find(|n| n.node_type == NodeType::Assistant && n.tool_name.is_some())
        .expect("no assistant tool call node found")
}

fn find_first_progress_type<'a>(nodes: &[&'a NodeResponse], contains: &str) -> &'a NodeResponse {
    nodes
        .iter()
        .find(|n| n.node_type == NodeType::Progress && n.summary.contains(contains))
        .unwrap_or_else(|| panic!("no progress node with summary containing '{}'", contains))
}

/// Snapshot: redact volatile fields (uuid, timestamp, data) so snapshots are stable.
fn redacted_json(node: &NodeResponse) -> serde_json::Value {
    let mut v = serde_json::to_value(node).unwrap();
    if let Some(obj) = v.as_object_mut() {
        obj.insert("uuid".into(), serde_json::json!("[uuid]"));
        obj.insert("timestamp".into(), serde_json::json!(0));
        obj.remove("data");
        obj.remove("children");
    }
    v
}

#[test]
fn snapshot_first_user_message() {
    let roots = build_all_responses();
    let all = collect_all(&roots);
    let node = find_first_by_type(&all, NodeType::User);
    insta::assert_json_snapshot!("first_user_message", redacted_json(node));
}

#[test]
fn snapshot_assistant_tool_use() {
    let roots = build_all_responses();
    let all = collect_all(&roots);
    let node = find_first_tool_call(&all);
    insta::assert_json_snapshot!("assistant_tool_use", redacted_json(node));
}

#[test]
fn snapshot_progress_hook() {
    let roots = build_all_responses();
    let all = collect_all(&roots);
    let node = find_first_progress_type(&all, "Hook:");
    insta::assert_json_snapshot!("progress_hook", redacted_json(node));
}

#[test]
fn snapshot_system_node() {
    let roots = build_all_responses();
    let all = collect_all(&roots);
    let node = find_first_by_type(&all, NodeType::System);
    insta::assert_json_snapshot!("system_node", redacted_json(node));
}

#[test]
fn snapshot_tree_metadata() {
    let roots = build_all_responses();
    fn count_nodes(nodes: &[NodeResponse]) -> usize {
        nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
    }
    fn max_depth(nodes: &[NodeResponse]) -> usize {
        nodes
            .iter()
            .map(|n| {
                if n.children.is_empty() {
                    n.depth
                } else {
                    max_depth(&n.children)
                }
            })
            .max()
            .unwrap_or(0)
    }

    let metadata = serde_json::json!({
        "total_nodes": count_nodes(&roots),
        "max_depth": max_depth(&roots),
        "root_count": roots.len(),
    });
    insta::assert_json_snapshot!("tree_metadata", metadata);
}
