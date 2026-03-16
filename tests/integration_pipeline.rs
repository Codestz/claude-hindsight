//! Integration tests: JSONL → parse → tree → NodeResponse end-to-end pipeline.
//!
//! Verifies the full transformation chain produces correct, well-formed output.

use hindsight::api::responses::{NodeResponse, NodeResponseContext, TreeResponse};
use hindsight::parser::models::NodeType;
use std::path::Path;

fn fixture_path() -> &'static Path {
    Path::new("tests/fixtures/full_session.jsonl")
}

fn build_tree_response() -> TreeResponse {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    let tree_roots = hindsight::build_simple_tree(session.nodes);
    let mut ctx = NodeResponseContext::new();
    let roots: Vec<NodeResponse> = tree_roots
        .iter()
        .map(|root| NodeResponse::from_tree_node_with_context(root, &mut ctx))
        .collect();

    let total_nodes = count_nodes(&roots);
    let max_depth = max_depth(&roots);

    TreeResponse {
        roots,
        total_nodes,
        max_depth,
    }
}

fn count_nodes(nodes: &[NodeResponse]) -> usize {
    nodes
        .iter()
        .map(|n| 1 + count_nodes(&n.children))
        .sum()
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

fn collect_all_nodes(roots: &[NodeResponse]) -> Vec<&NodeResponse> {
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

// ── Structural assertions ────────────────────────────────────────────────────

#[test]
fn root_count_is_reasonable() {
    let resp = build_tree_response();
    assert!(resp.roots.len() > 0, "must have at least 1 root");
    assert!(
        resp.roots.len() < 83,
        "root count {} should be less than raw line count",
        resp.roots.len()
    );
}

#[test]
fn max_depth_within_bounds() {
    let resp = build_tree_response();
    assert!(
        resp.max_depth < 20,
        "max depth {} should be < 20",
        resp.max_depth
    );
}

#[test]
fn no_empty_labels() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);
    for node in &all {
        assert!(
            !node.label.is_empty(),
            "node {:?} (type {:?}) has empty label",
            node.uuid,
            node.node_type
        );
    }
}

#[test]
fn no_empty_summaries_on_content_nodes() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);
    let content_types = [NodeType::User, NodeType::Assistant, NodeType::Progress, NodeType::System];

    for node in &all {
        if content_types.contains(&node.node_type) {
            assert!(
                !node.summary.is_empty(),
                "node {:?} (type {:?}) has empty summary",
                node.uuid,
                node.node_type
            );
        }
    }
}

#[test]
fn error_nodes_have_has_error_true() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);

    let error_count = all.iter().filter(|n| n.has_error).count();
    assert!(
        error_count >= 1,
        "expected at least 1 error node in fixture"
    );
}

#[test]
fn token_usage_present_on_assistant_nodes() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);

    let assistant_nodes: Vec<_> = all
        .iter()
        .filter(|n| n.node_type == NodeType::Assistant)
        .collect();
    assert!(!assistant_nodes.is_empty(), "must have assistant nodes");

    let with_usage = assistant_nodes
        .iter()
        .filter(|n| n.token_usage.is_some())
        .count();
    assert!(
        with_usage > 0,
        "at least some assistant nodes should have token_usage"
    );
}

#[test]
fn tool_names_on_tool_call_nodes() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);

    let with_tool_name: Vec<_> = all.iter().filter(|n| n.tool_name.is_some()).collect();
    assert!(
        !with_tool_name.is_empty(),
        "at least some nodes should have tool_name"
    );

    // Verify known tool names
    let names: Vec<&str> = with_tool_name
        .iter()
        .filter_map(|n| n.tool_name.as_deref())
        .collect();
    assert!(
        names.iter().any(|n| *n == "Read" || *n == "Bash" || *n == "Edit" || *n == "Glob" || *n == "Grep"),
        "expected common tool names, got {:?}",
        names
    );
}

#[test]
fn children_depth_equals_parent_depth_plus_one() {
    let resp = build_tree_response();
    fn check_depths(nodes: &[NodeResponse], expected_depth: usize) {
        for node in nodes {
            assert_eq!(
                node.depth, expected_depth,
                "node {:?} has depth {} but expected {}",
                node.uuid, node.depth, expected_depth
            );
            check_depths(&node.children, expected_depth + 1);
        }
    }
    check_depths(&resp.roots, 0);
}

#[test]
fn sse_dedup_reduced_raw_count() {
    let session = hindsight::parse_session(fixture_path()).unwrap();
    // Raw fixture has 83 lines, after SSE merge + progress dedup should be less
    assert!(
        session.nodes.len() < 83,
        "dedup should reduce node count below 83, got {}",
        session.nodes.len()
    );
}

#[test]
fn total_nodes_matches_actual_count() {
    let resp = build_tree_response();
    let actual = count_nodes(&resp.roots);
    assert_eq!(resp.total_nodes, actual);
}

#[test]
fn file_paths_on_file_tool_nodes() {
    let resp = build_tree_response();
    let all = collect_all_nodes(&resp.roots);

    let with_file_paths: Vec<_> = all.iter().filter(|n| !n.file_paths.is_empty()).collect();
    // The fixture has Read/Edit/Write tool calls with file_path inputs
    assert!(
        !with_file_paths.is_empty(),
        "at least some nodes should have file_paths"
    );
}
