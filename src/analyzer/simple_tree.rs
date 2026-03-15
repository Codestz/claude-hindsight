//! Simple tree builder - builds hierarchical tree from parent_uuid relationships
//!
//! Clean, maintainable approach without complex grouping logic.

use crate::analyzer::TreeNode;
use crate::parser::models::NodeType;
use crate::parser::ExecutionNode;
use std::collections::HashMap;
use std::rc::Rc;

/// Deduplicate agent progress nodes by `agentId`, keeping the FIRST frame.
///
/// After `dedup_progress_by_tool_use_id` in `transcript.rs` collapsed SSE
/// frames per toolUseID, many progress nodes remain because each Agent tool
/// call gets its own toolUseID. This pass collapses them to one per agent
/// worker so the tree stays clean.
///
/// **Why FIRST?** The first frame carries the agent prompt (the task
/// description); later frames have it empty or stale. The prompt is the
/// most useful label for the tree display.
fn dedup_agent_progress_by_agent_id(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    use std::collections::HashMap;

    // First pass: record the index of the FIRST agent_progress per agentId.
    // The first frame carries the agent prompt; later frames have it empty.
    let mut first_idx: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.node_type == NodeType::Progress {
            if let Some(data) = node.extra.as_ref().and_then(|e| e.get("data")) {
                if data.get("type").and_then(|t| t.as_str()) == Some("agent_progress") {
                    if let Some(agent_id) = data.get("agentId").and_then(|a| a.as_str()) {
                        first_idx.entry(agent_id.to_string()).or_insert(i);
                    }
                }
            }
        }
    }

    // Second pass: keep non-agent-progress unchanged; for agent_progress keep
    // only the first occurrence per agentId (has the prompt text).
    nodes
        .into_iter()
        .enumerate()
        .filter(|(i, node)| {
            if node.node_type == NodeType::Progress {
                if let Some(data) = node.extra.as_ref().and_then(|e| e.get("data")) {
                    if data.get("type").and_then(|t| t.as_str()) == Some("agent_progress") {
                        if let Some(agent_id) = data.get("agentId").and_then(|a| a.as_str()) {
                            return first_idx.get(agent_id) == Some(i);
                        }
                    }
                }
            }
            true
        })
        .map(|(_, node)| node)
        .collect()
}

/// Returns true if this user node is a local command (slash command, stdout, caveat)
/// rather than a real user message. These are injected by the Claude Code client
/// and should be hidden from the execution tree.
fn is_local_command_node(node: &ExecutionNode) -> bool {
    if node.node_type != NodeType::User {
        return false;
    }
    let text = match node.message.as_ref() {
        Some(m) => m.text_content(),
        None => return false,
    };
    crate::analyzer::prompt_detect::is_local_command_text(&text)
}

/// Filter out local command nodes (slash commands, stdout, caveats)
fn filter_local_commands(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    nodes.into_iter().filter(|n| !is_local_command_node(n)).collect()
}

/// Build a simple parent-child tree from flat nodes
pub fn build_simple_tree(nodes: Vec<ExecutionNode>) -> Vec<TreeNode> {
    // Filter out local command nodes (slash commands, stdout, caveats)
    let nodes = filter_local_commands(nodes);

    // Deduplicate progress nodes (collapse consecutive agent progress updates)
    let nodes = dedup_agent_progress_by_agent_id(nodes);

    // Wrap all nodes in Rc immediately to avoid expensive cloning
    let rc_nodes: Vec<Rc<ExecutionNode>> = nodes.into_iter().map(Rc::new).collect();

    // Index nodes by UUID for fast lookup
    let mut node_map: HashMap<String, Rc<ExecutionNode>> = HashMap::new();
    let mut children_map: HashMap<String, Vec<Rc<ExecutionNode>>> = HashMap::new();
    let mut root_nodes: Vec<Rc<ExecutionNode>> = Vec::new();

    // First pass: index all nodes
    for rc_node in rc_nodes {
        if let Some(ref uuid) = rc_node.uuid {
            node_map.insert(uuid.clone(), rc_node);
        }
    }

    // Second pass: build parent-child relationships.
    // If a node's parent was removed by dedup, promote it to a root rather
    // than letting it disappear into an unreferenced children_map entry.
    for rc_node in node_map.values() {
        match rc_node.parent_uuid.as_deref() {
            Some(pid) if node_map.contains_key(pid) => {
                // Parent exists — attach as child
                children_map
                    .entry(pid.to_string())
                    .or_default()
                    .push(Rc::clone(rc_node));
            }
            _ => {
                // No parent, or parent was removed by dedup → root
                root_nodes.push(Rc::clone(rc_node));
            }
        }
    }

    // Sort all children by timestamp
    for children in children_map.values_mut() {
        children.sort_by(|a, b| {
            let ts_a = a.timestamp.unwrap_or(0);
            let ts_b = b.timestamp.unwrap_or(0);
            ts_a.cmp(&ts_b)
        });
    }

    // Filter out childless progress roots — these are typically orphans from
    // dedup that were promoted to roots. They carry no conversational content
    // and just clutter the tree. Progress nodes that DO have children (e.g.
    // the SessionStart hook whose child is the first user message) are kept.
    root_nodes.retain(|n| {
        if n.node_type == NodeType::Progress {
            // Keep if it has children
            if let Some(ref uuid) = n.uuid {
                return children_map.contains_key(uuid);
            }
            return false;
        }
        true
    });

    // Sort root nodes by timestamp
    root_nodes.sort_by(|a, b| {
        let ts_a = a.timestamp.unwrap_or(0);
        let ts_b = b.timestamp.unwrap_or(0);
        ts_a.cmp(&ts_b)
    });

    // Third pass: build TreeNode hierarchy from roots
    root_nodes
        .into_iter()
        .map(|rc_node| build_tree_node(&rc_node, &children_map, 0))
        .collect()
}

/// Recursively build TreeNode from ExecutionNode
fn build_tree_node(
    rc_node: &Rc<ExecutionNode>,
    children_map: &HashMap<String, Vec<Rc<ExecutionNode>>>,
    depth: usize,
) -> TreeNode {
    let children = if let Some(ref uuid) = rc_node.uuid {
        if let Some(child_nodes) = children_map.get(uuid) {
            child_nodes
                .iter()
                .map(|child_rc| build_tree_node(child_rc, children_map, depth + 1))
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    TreeNode {
        node: Rc::clone(rc_node),
        children,
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ExecutionNode;

    fn create_test_node(uuid: &str, parent_uuid: Option<&str>, node_type: NodeType) -> ExecutionNode {
        ExecutionNode {
            uuid: Some(uuid.to_string()),
            parent_uuid: parent_uuid.map(|s| s.to_string()),
            timestamp: Some(1000),
            node_type,
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
    fn test_build_simple_tree_single_node() {
        let nodes = vec![create_test_node("root", None, NodeType::User)];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].children.len(), 0);
        assert_eq!(tree[0].node.node_type, NodeType::User);
    }

    #[test]
    fn test_build_simple_tree_with_children() {
        let nodes = vec![
            create_test_node("root", None, NodeType::User),
            create_test_node("child1", Some("root"), NodeType::Assistant),
            create_test_node("child2", Some("root"), NodeType::Unknown),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].depth, 1);
        assert_eq!(tree[0].children[1].depth, 1);
    }

    #[test]
    fn test_build_simple_tree_deep_hierarchy() {
        let nodes = vec![
            create_test_node("root", None, NodeType::User),
            create_test_node("level1", Some("root"), NodeType::Assistant),
            create_test_node("level2", Some("level1"), NodeType::Unknown),
            create_test_node("level3", Some("level2"), NodeType::Unknown),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].children[0].depth, 1);
        assert_eq!(tree[0].children[0].children[0].depth, 2);
        assert_eq!(tree[0].children[0].children[0].children[0].depth, 3);
    }

    #[test]
    fn test_build_simple_tree_multiple_roots() {
        let nodes = vec![
            create_test_node("root1", None, NodeType::User),
            create_test_node("root2", None, NodeType::Assistant),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[1].depth, 0);
    }

    #[test]
    fn test_rc_sharing() {
        // Test that Rc is properly used (nodes are shared, not cloned)
        let nodes = vec![
            create_test_node("root", None, NodeType::User),
            create_test_node("child", Some("root"), NodeType::Assistant),
        ];

        let tree = build_simple_tree(nodes);

        // The Rc should have a strong count of 1 (only the TreeNode holds it)
        assert_eq!(Rc::strong_count(&tree[0].node), 1);
        assert_eq!(Rc::strong_count(&tree[0].children[0].node), 1);
    }

    // ── Orphan promotion tests ────────────────────────────────────

    #[test]
    fn test_orphan_nodes_promoted_to_roots() {
        // Child "orphan" references parent "deleted" which is NOT in the node list.
        // The orphan must become a root rather than disappearing.
        let nodes = vec![
            create_test_node("root", None, NodeType::User),
            create_test_node("orphan", Some("deleted"), NodeType::Assistant),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 2, "orphan must be promoted to root");
        let types: Vec<NodeType> = tree.iter().map(|t| t.node.node_type).collect();
        assert!(types.contains(&NodeType::User));
        assert!(types.contains(&NodeType::Assistant));
    }

    #[test]
    fn test_orphan_with_own_children_preserved() {
        // "orphan" references missing parent, but "grandchild" references "orphan".
        // Both must appear: orphan as root, grandchild as its child.
        let nodes = vec![
            create_test_node("orphan", Some("deleted"), NodeType::Assistant),
            create_test_node("grandchild", Some("orphan"), NodeType::User),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1, "orphan is the only root");
        assert_eq!(tree[0].node.node_type, NodeType::Assistant);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].node.node_type, NodeType::User);
    }

    // ── Childless progress root filtering ─────────────────────────

    #[test]
    fn test_childless_progress_roots_are_filtered() {
        let nodes = vec![
            create_test_node("user-root", None, NodeType::User),
            create_test_node("prog-noise", None, NodeType::Progress),
        ];

        let tree = build_simple_tree(nodes);

        // The childless progress root should be filtered out
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].node.node_type, NodeType::User);
    }

    #[test]
    fn test_progress_root_with_children_is_kept() {
        let nodes = vec![
            create_test_node("prog-root", None, NodeType::Progress),
            create_test_node("child", Some("prog-root"), NodeType::User),
        ];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1, "progress root with children must be kept");
        assert_eq!(tree[0].node.node_type, NodeType::Progress);
        assert_eq!(tree[0].children.len(), 1);
    }

    // ── Agent progress dedup ──────────────────────────────────────

    fn create_agent_progress_node(uuid: &str, agent_id: &str, prompt: &str) -> ExecutionNode {
        let mut extra = std::collections::HashMap::new();
        extra.insert(
            "data".to_string(),
            serde_json::json!({
                "type": "agent_progress",
                "agentId": agent_id,
                "prompt": prompt,
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
    fn test_agent_progress_dedup_keeps_first_per_agent_id() {
        let nodes = vec![
            create_agent_progress_node("a1", "agent-X", "Explore code"),
            create_test_node("user1", None, NodeType::User),
            create_agent_progress_node("a2", "agent-X", ""),
            create_agent_progress_node("a3", "agent-X", ""),
            create_agent_progress_node("b1", "agent-Y", "Run tests"),
        ];

        let deduped = dedup_agent_progress_by_agent_id(nodes);

        // 3 agent-X nodes → 1 (first), 1 agent-Y → 1, 1 user → 1 = 3 total
        let agent_progress: Vec<&ExecutionNode> = deduped
            .iter()
            .filter(|n| n.node_type == NodeType::Progress)
            .collect();
        assert_eq!(agent_progress.len(), 2, "one per agentId");
        assert_eq!(agent_progress[0].uuid.as_deref(), Some("a1"), "first agent-X kept");
        assert_eq!(agent_progress[1].uuid.as_deref(), Some("b1"), "first agent-Y kept");
    }

    #[test]
    fn test_agent_progress_dedup_preserves_non_agent_progress() {
        let mut hook_extra = std::collections::HashMap::new();
        hook_extra.insert(
            "data".to_string(),
            serde_json::json!({ "type": "hook_progress", "hookName": "SessionStart" }),
        );
        let hook_node = ExecutionNode {
            uuid: Some("hook1".to_string()),
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
            extra: Some(hook_extra),
        };

        let nodes = vec![
            hook_node,
            create_agent_progress_node("a1", "agent-X", "task"),
            create_agent_progress_node("a2", "agent-X", ""),
        ];

        let deduped = dedup_agent_progress_by_agent_id(nodes);

        assert_eq!(deduped.len(), 2, "hook_progress preserved, agent-X collapsed to 1");
        assert_eq!(deduped[0].uuid.as_deref(), Some("hook1"));
        assert_eq!(deduped[1].uuid.as_deref(), Some("a1"));
    }
}
