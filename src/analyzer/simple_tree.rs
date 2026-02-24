//! Simple tree builder - builds hierarchical tree from parent_uuid relationships
//!
//! Clean, maintainable approach without complex grouping logic.

use crate::analyzer::TreeNode;
use crate::parser::ExecutionNode;
use std::collections::HashMap;
use std::rc::Rc;

/// Deduplicate consecutive progress nodes (especially agent progress)
fn deduplicate_progress(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    let mut result = Vec::new();
    let mut last_agent_id: Option<String> = None;
    let mut skip_count = 0;

    for node in nodes {
        // Check if this is agent progress
        if node.node_type == "progress" {
            if let Some(data) = node.extra.as_ref().and_then(|e| e.get("data")) {
                if let Some(progress_type) = data.get("type").and_then(|t| t.as_str()) {
                    if progress_type == "agent_progress" {
                        if let Some(agent_id) = data.get("agentId").and_then(|a| a.as_str()) {
                            // If same agent as last progress, skip it
                            if last_agent_id.as_deref() == Some(agent_id) {
                                skip_count += 1;
                                continue;
                            } else {
                                // Different agent, update tracker
                                if skip_count > 0 {
                                    // Add a summary node showing how many were skipped
                                    // (we'll just reset the counter for now)
                                    skip_count = 0;
                                }
                                last_agent_id = Some(agent_id.to_string());
                            }
                        }
                    } else {
                        // Not agent progress, reset tracker
                        last_agent_id = None;
                        skip_count = 0;
                    }
                }
            }
        } else {
            // Not a progress node, reset tracker
            last_agent_id = None;
            skip_count = 0;
        }

        result.push(node);
    }

    result
}

/// Returns true if this user node is a local command (slash command, stdout, caveat)
/// rather than a real user message. These are injected by the Claude Code client
/// and should be hidden from the execution tree.
fn is_local_command_node(node: &ExecutionNode) -> bool {
    if node.node_type != "user" {
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
    let nodes = deduplicate_progress(nodes);

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

    // Second pass: build parent-child relationships
    for rc_node in node_map.values() {
        if let Some(ref parent_uuid) = rc_node.parent_uuid {
            // Has parent - add to children map (Rc::clone is just a pointer increment)
            children_map
                .entry(parent_uuid.clone())
                .or_default()
                .push(Rc::clone(rc_node));
        } else {
            // No parent - this is a root
            root_nodes.push(Rc::clone(rc_node));
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

    fn create_test_node(uuid: &str, parent_uuid: Option<&str>, node_type: &str) -> ExecutionNode {
        ExecutionNode {
            uuid: Some(uuid.to_string()),
            parent_uuid: parent_uuid.map(|s| s.to_string()),
            timestamp: Some(1000),
            node_type: node_type.to_string(),
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
        let nodes = vec![create_test_node("root", None, "user")];

        let tree = build_simple_tree(nodes);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].children.len(), 0);
        assert_eq!(tree[0].node.node_type, "user");
    }

    #[test]
    fn test_build_simple_tree_with_children() {
        let nodes = vec![
            create_test_node("root", None, "user"),
            create_test_node("child1", Some("root"), "assistant"),
            create_test_node("child2", Some("root"), "tool_use"),
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
            create_test_node("root", None, "user"),
            create_test_node("level1", Some("root"), "assistant"),
            create_test_node("level2", Some("level1"), "thinking"),
            create_test_node("level3", Some("level2"), "tool_use"),
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
            create_test_node("root1", None, "user"),
            create_test_node("root2", None, "assistant"),
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
            create_test_node("root", None, "user"),
            create_test_node("child", Some("root"), "assistant"),
        ];

        let tree = build_simple_tree(nodes);

        // The Rc should have a strong count of 1 (only the TreeNode holds it)
        assert_eq!(Rc::strong_count(&tree[0].node), 1);
        assert_eq!(Rc::strong_count(&tree[0].children[0].node), 1);
    }
}
