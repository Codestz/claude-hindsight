//! Simple tree builder - builds hierarchical tree from parent_uuid relationships
//!
//! Clean, maintainable approach without complex grouping logic.

use crate::analyzer::TreeNode;
use crate::parser::ExecutionNode;
use std::collections::HashMap;

/// Deduplicate consecutive progress nodes (especially agent progress)
fn deduplicate_progress(nodes: Vec<ExecutionNode>) -> Vec<ExecutionNode> {
    let mut result = Vec::new();
    let mut last_agent_id: Option<String> = None;
    let mut skip_count = 0;

    for node in nodes {
        // Check if this is agent progress
        if node.node_type == "progress" {
            if let Some(data) = node.extra.get("data") {
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

/// Build a simple parent-child tree from flat nodes
pub fn build_simple_tree(nodes: Vec<ExecutionNode>) -> Vec<TreeNode> {
    // Deduplicate progress nodes (collapse consecutive agent progress updates)
    let nodes = deduplicate_progress(nodes);

    // Index nodes by UUID for fast lookup
    let mut node_map: HashMap<String, ExecutionNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<ExecutionNode>> = HashMap::new();
    let mut root_nodes: Vec<ExecutionNode> = Vec::new();

    // First pass: index all nodes
    for node in nodes {
        if let Some(ref uuid) = node.uuid {
            node_map.insert(uuid.clone(), node);
        }
    }

    // Second pass: build parent-child relationships
    for (_uuid, node) in &node_map {
        if let Some(ref parent_uuid) = node.parent_uuid {
            // Has parent - add to children map
            children_map
                .entry(parent_uuid.clone())
                .or_insert_with(Vec::new)
                .push(node.clone());
        } else {
            // No parent - this is a root
            root_nodes.push(node.clone());
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
        .map(|node| build_tree_node(&node, &children_map, 0))
        .collect()
}

/// Recursively build TreeNode from ExecutionNode
fn build_tree_node(
    node: &ExecutionNode,
    children_map: &HashMap<String, Vec<ExecutionNode>>,
    depth: usize,
) -> TreeNode {
    let children = if let Some(ref uuid) = node.uuid {
        if let Some(child_nodes) = children_map.get(uuid) {
            child_nodes
                .iter()
                .map(|child| build_tree_node(child, children_map, depth + 1))
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    TreeNode {
        node: node.clone(),
        children,
        depth,
    }
}
