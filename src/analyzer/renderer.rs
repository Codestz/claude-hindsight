//! Tree rendering for terminal output
//!
//! Provides ASCII tree visualization for execution trees.

use crate::analyzer::{ExecutionTree, TreeNode};

/// Configuration for tree rendering
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Maximum depth to render (None = unlimited)
    pub max_depth: Option<usize>,

    /// Maximum nodes to render (None = unlimited)
    pub max_nodes: Option<usize>,

    /// Show timestamps
    pub show_timestamps: bool,

    /// Show node types
    pub show_types: bool,

    /// Compact mode (less spacing)
    pub compact: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        RenderConfig {
            max_depth: Some(10),
            max_nodes: Some(100),
            show_timestamps: false,
            show_types: false,
            compact: false,
        }
    }
}

/// Render an execution tree to a string
pub fn render_tree(tree: &ExecutionTree, config: &RenderConfig) -> String {
    let mut output = String::new();
    let mut node_count = 0;

    for (i, root) in tree.roots.iter().enumerate() {
        if let Some(max) = config.max_nodes {
            if node_count >= max {
                output.push_str(&format!(
                    "\n... ({} more root nodes)\n",
                    tree.roots.len() - i
                ));
                break;
            }
        }

        render_node(root, &mut output, "", true, config, &mut node_count);

        if !config.compact && i < tree.roots.len() - 1 {
            output.push('\n');
        }
    }

    output
}

/// Recursively render a tree node
fn render_node(
    node: &TreeNode,
    output: &mut String,
    prefix: &str,
    is_last: bool,
    config: &RenderConfig,
    node_count: &mut usize,
) {
    // Check depth limit
    if let Some(max_depth) = config.max_depth {
        if node.depth >= max_depth {
            return;
        }
    }

    // Check node count limit
    if let Some(max_nodes) = config.max_nodes {
        if *node_count >= max_nodes {
            return;
        }
    }

    *node_count += 1;

    // Render current node
    let branch = if is_last { "└─" } else { "├─" };
    let summary = node.summary();

    output.push_str(prefix);
    output.push_str(branch);
    output.push(' ');
    output.push_str(&summary);

    // Add type if requested
    if config.show_types {
        output.push_str(&format!(" [{}]", node.node.node_type));
    }

    // Add timestamp if requested
    if config.show_timestamps {
        if let Some(timestamp) = node.timestamp() {
            let dt = chrono::DateTime::from_timestamp_millis(timestamp);
            if let Some(dt) = dt {
                output.push_str(&format!(
                    " {}",
                    dt.format("%H:%M:%S")
                ));
            }
        }
    }

    output.push('\n');

    // Render children
    let child_prefix = format!(
        "{}{}",
        prefix,
        if is_last { "   " } else { "│  " }
    );

    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        render_node(child, output, &child_prefix, is_last_child, config, node_count);
    }
}

/// Render a flat list of nodes (for chronological view)
pub fn render_flat(tree: &ExecutionTree, config: &RenderConfig) -> String {
    let mut output = String::new();
    let nodes = tree.depth_first_traversal();

    let max_nodes = config.max_nodes.unwrap_or(nodes.len());

    for (i, node) in nodes.iter().enumerate().take(max_nodes) {
        // Add timestamp prefix
        if config.show_timestamps {
            if let Some(timestamp) = node.timestamp() {
                let dt = chrono::DateTime::from_timestamp_millis(timestamp);
                if let Some(dt) = dt {
                    output.push_str(&format!("[{}] ", dt.format("%H:%M:%S")));
                }
            } else {
                output.push_str("[--:--:--] ");
            }
        }

        // Add depth indicator
        output.push_str(&"  ".repeat(node.depth));

        // Add summary
        output.push_str(&node.summary());

        if config.show_types {
            output.push_str(&format!(" [{}]", node.node.node_type));
        }

        output.push('\n');
    }

    if max_nodes < nodes.len() {
        output.push_str(&format!("\n... ({} more nodes)\n", nodes.len() - max_nodes));
    }

    output
}

/// Render tree statistics
pub fn render_stats(tree: &ExecutionTree) -> String {
    let mut output = String::new();

    output.push_str("📊 Tree Statistics\n\n");
    output.push_str(&format!("   Total nodes: {}\n", tree.stats.total_nodes));
    output.push_str(&format!("   Root nodes: {}\n", tree.roots.len()));
    output.push_str(&format!("   Max depth: {}\n", tree.calculate_max_depth()));
    output.push_str(&format!("   Tool calls: {}\n", tree.stats.tool_calls));
    output.push_str(&format!("   Errors: {}\n", tree.stats.errors));
    output.push_str(&format!("   Thinking blocks: {}\n", tree.stats.thinking_blocks));

    output.push_str("\n📋 Nodes by Type:\n");
    let mut types: Vec<_> = tree.stats.nodes_by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));

    for (node_type, count) in types.iter().take(10) {
        output.push_str(&format!("   {} - {} nodes\n", node_type, count));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ExecutionNode;
    use std::collections::HashMap;

    #[test]
    fn test_render_empty_tree() {
        let tree = ExecutionTree::from_nodes(vec![]);
        let config = RenderConfig::default();
        let output = render_tree(&tree, &config);
        assert_eq!(output, "");
    }

    #[test]
    fn test_render_single_node() {
        let node = ExecutionNode {
            uuid: Some("root".to_string()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: "user".to_string(),
            message: None,
            tool_use: None,
            tool_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: HashMap::new(),
        };

        let tree = ExecutionTree::from_nodes(vec![node]);
        let config = RenderConfig::default();
        let output = render_tree(&tree, &config);
        assert!(output.contains("👤 User"));
    }
}
