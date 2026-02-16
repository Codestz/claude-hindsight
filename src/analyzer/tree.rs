//! Execution tree builder
//!
//! Transforms flat JSONL nodes into hierarchical tree structure for visualization.

use crate::parser::ExecutionNode;
use std::collections::HashMap;

/// A node in the execution tree with children
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// The execution node data
    pub node: ExecutionNode,

    /// Child nodes
    pub children: Vec<TreeNode>,

    /// Depth in the tree (0 = root)
    pub depth: usize,
}

/// Statistics about the execution tree
#[derive(Debug, Clone)]
pub struct TreeStats {
    /// Total number of nodes
    pub total_nodes: usize,

    /// Maximum depth of the tree
    pub max_depth: usize,

    /// Nodes by type
    pub nodes_by_type: HashMap<String, usize>,

    /// Total tool calls
    pub tool_calls: usize,

    /// Total errors
    pub errors: usize,

    /// Total thinking blocks
    pub thinking_blocks: usize,
}

/// Hierarchical execution tree
#[derive(Debug)]
pub struct ExecutionTree {
    /// Root nodes (nodes with no parent)
    pub roots: Vec<TreeNode>,

    /// Tree statistics
    pub stats: TreeStats,
}

impl ExecutionTree {
    /// Build a tree from flat list of execution nodes
    ///
    /// Uses uuid/parent_uuid to establish parent-child relationships.
    pub fn from_nodes(nodes: Vec<ExecutionNode>) -> Self {
        // Build UUID to node index mapping
        let mut uuid_to_index: HashMap<String, usize> = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            if let Some(ref uuid) = node.uuid {
                uuid_to_index.insert(uuid.clone(), i);
            }
        }

        // Build parent-to-children mapping
        let mut parent_to_children: HashMap<Option<String>, Vec<usize>> = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            parent_to_children
                .entry(node.parent_uuid.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }

        // Build tree recursively starting from roots (nodes with no parent)
        let roots = if let Some(root_indices) = parent_to_children.get(&None) {
            root_indices
                .iter()
                .map(|&i| Self::build_tree_node(&nodes, i, &parent_to_children, 0))
                .collect()
        } else {
            Vec::new()
        };

        // Calculate statistics
        let stats = Self::calculate_stats(&nodes);

        ExecutionTree { roots, stats }
    }

    /// Recursively build a tree node and its children
    fn build_tree_node(
        nodes: &[ExecutionNode],
        index: usize,
        parent_to_children: &HashMap<Option<String>, Vec<usize>>,
        depth: usize,
    ) -> TreeNode {
        let node = nodes[index].clone();
        let node_uuid = node.uuid.clone();

        // Get children for this node
        let children = if let Some(ref uuid) = node_uuid {
            if let Some(child_indices) = parent_to_children.get(&Some(uuid.clone())) {
                child_indices
                    .iter()
                    .map(|&i| Self::build_tree_node(nodes, i, parent_to_children, depth + 1))
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        TreeNode {
            node,
            children,
            depth,
        }
    }

    /// Calculate tree statistics
    fn calculate_stats(nodes: &[ExecutionNode]) -> TreeStats {
        let mut nodes_by_type: HashMap<String, usize> = HashMap::new();
        let mut tool_calls = 0;
        let mut errors = 0;
        let mut thinking_blocks = 0;

        for node in nodes {
            // Count by type
            *nodes_by_type.entry(node.node_type.clone()).or_insert(0) += 1;

            // Count tool calls
            if node.tool_use.is_some() {
                tool_calls += 1;
            }

            // Count errors
            if let Some(ref result) = node.tool_result {
                if result.is_error.unwrap_or(false) {
                    errors += 1;
                }
            }

            // Count thinking blocks
            if node.thinking.is_some() {
                thinking_blocks += 1;
            }
        }

        TreeStats {
            total_nodes: nodes.len(),
            max_depth: 0, // Will be calculated during traversal if needed
            nodes_by_type,
            tool_calls,
            errors,
            thinking_blocks,
        }
    }

    /// Get all nodes in depth-first order
    pub fn depth_first_traversal(&self) -> Vec<&TreeNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            Self::traverse_depth_first(root, &mut result);
        }
        result
    }

    /// Recursively traverse tree in depth-first order
    fn traverse_depth_first<'a>(node: &'a TreeNode, result: &mut Vec<&'a TreeNode>) {
        result.push(node);
        for child in &node.children {
            Self::traverse_depth_first(child, result);
        }
    }

    /// Get all nodes in breadth-first order
    pub fn breadth_first_traversal(&self) -> Vec<&TreeNode> {
        let mut result = Vec::new();
        let mut queue: Vec<&TreeNode> = self.roots.iter().collect();

        while let Some(node) = queue.first() {
            result.push(*node);
            queue.extend(node.children.iter());
            queue.remove(0);
        }

        result
    }

    /// Find a node by UUID
    pub fn find_by_uuid(&self, uuid: &str) -> Option<&TreeNode> {
        for root in &self.roots {
            if let Some(node) = Self::find_node_by_uuid(root, uuid) {
                return Some(node);
            }
        }
        None
    }

    /// Recursively search for node by UUID
    fn find_node_by_uuid<'a>(node: &'a TreeNode, uuid: &str) -> Option<&'a TreeNode> {
        if let Some(ref node_uuid) = node.node.uuid {
            if node_uuid == uuid {
                return Some(node);
            }
        }

        for child in &node.children {
            if let Some(found) = Self::find_node_by_uuid(child, uuid) {
                return Some(found);
            }
        }

        None
    }

    /// Get all tool calls in chronological order
    pub fn get_tool_calls(&self) -> Vec<&TreeNode> {
        self.depth_first_traversal()
            .into_iter()
            .filter(|node| node.node.tool_use.is_some())
            .collect()
    }

    /// Get all errors in chronological order
    pub fn get_errors(&self) -> Vec<&TreeNode> {
        self.depth_first_traversal()
            .into_iter()
            .filter(|node| {
                node.node
                    .tool_result
                    .as_ref()
                    .and_then(|r| r.is_error)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all thinking blocks in chronological order
    pub fn get_thinking_blocks(&self) -> Vec<&TreeNode> {
        self.depth_first_traversal()
            .into_iter()
            .filter(|node| node.node.thinking.is_some())
            .collect()
    }

    /// Calculate the maximum depth of the tree
    pub fn calculate_max_depth(&self) -> usize {
        self.roots
            .iter()
            .map(|root| Self::get_subtree_depth(root))
            .max()
            .unwrap_or(0)
    }

    /// Get the depth of a subtree
    fn get_subtree_depth(node: &TreeNode) -> usize {
        if node.children.is_empty() {
            node.depth
        } else {
            node.children
                .iter()
                .map(Self::get_subtree_depth)
                .max()
                .unwrap_or(node.depth)
        }
    }
}

impl TreeNode {
    /// Get a display-friendly label for this node (Nerd Font icons)
    pub fn label(&self) -> String {
        match self.node.node_type.as_str() {
            "group" => {
                // Synthetic group node - extract label from message content
                if let Some(ref msg) = self.node.message {
                    if let Some(ref content) = msg.content {
                        if let Some(label) = content.as_str() {
                            return label.to_string();
                        }
                    }
                }
                " Group".to_string()
            }
            "tool_use" => {
                if let Some(ref tool_use) = self.node.tool_use {
                    format!(" {}", tool_use.name)
                } else {
                    " Tool".to_string()
                }
            }
            "tool_result" => {
                if let Some(ref result) = self.node.tool_result {
                    if result.is_error.unwrap_or(false) {
                        " Error".to_string()
                    } else {
                        " Success".to_string()
                    }
                } else {
                    " Result".to_string()
                }
            }
            "thinking" => " Thinking".to_string(),
            "user" => " User".to_string(),
            "assistant" => " Assistant".to_string(),
            "message" => {
                if let Some(ref msg) = self.node.message {
                    match msg.role.as_deref() {
                        Some("user") => " User".to_string(),
                        Some("assistant") => " Assistant".to_string(),
                        _ => " Message".to_string(),
                    }
                } else {
                    " Message".to_string()
                }
            }
            "progress" => " Progress".to_string(),
            "system" => " System".to_string(),
            "file-history-snapshot" => " File Snapshot".to_string(),
            _ => format!(" {}", self.node.node_type),
        }
    }

    /// Get a one-line summary of this node
    pub fn summary(&self) -> String {
        let label = self.label();

        // For user/assistant messages, show preview of content
        if self.node.node_type == "user" || self.node.node_type == "assistant" {
            if let Some(ref msg) = self.node.message {
                if let Some(ref content) = msg.content {
                    let preview = extract_text_preview(content, 60);
                    if !preview.is_empty() {
                        return format!("{}: {}", label, preview);
                    }
                }
            }
        }

        // For group nodes, just return the label
        if self.node.node_type == "group" {
            return label;
        }

        // For tool uses, show the tool name with parameters
        if let Some(ref tool_use) = self.node.tool_use {
            return format!("🔧 {}", tool_use.name);
        }

        // Add duration if available
        if let Some(ref result) = self.node.tool_result {
            if let Some(duration_ms) = result.duration_ms {
                return format!("{} ({}.{}s)", label, duration_ms / 1000, duration_ms % 1000 / 100);
            }
        }

        // Add token count if available
        if let Some(ref usage) = self.node.token_usage {
            let total = usage.input_tokens.unwrap_or(0) + usage.output_tokens.unwrap_or(0);
            if total > 0 {
                return format!("{} ({} tokens)", label, total);
            }
        }

        label
    }

    /// Check if this node has errors
    pub fn has_error(&self) -> bool {
        self.node
            .tool_result
            .as_ref()
            .and_then(|r| r.is_error)
            .unwrap_or(false)
    }

    /// Get the timestamp of this node in milliseconds
    pub fn timestamp(&self) -> Option<i64> {
        self.node.timestamp
    }
}

/// Extract text preview from message content (handles string or array format)
fn extract_text_preview(content: &serde_json::Value, max_len: usize) -> String {
    let text = match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            // Extract text from content blocks
            arr.iter()
                .filter_map(|block| {
                    if let Some(text) = block.get("text") {
                        text.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        _ => String::new(),
    };

    // Truncate and clean
    let cleaned = text.replace('\n', " ").trim().to_string();

    // Use char_indices to safely truncate at character boundaries (not byte boundaries)
    if cleaned.chars().count() > max_len {
        let truncate_pos = cleaned
            .char_indices()
            .nth(max_len)
            .map(|(idx, _)| idx)
            .unwrap_or(cleaned.len());
        format!("{}...", &cleaned[..truncate_pos])
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = ExecutionTree::from_nodes(vec![]);
        assert_eq!(tree.roots.len(), 0);
        assert_eq!(tree.stats.total_nodes, 0);
    }

    #[test]
    fn test_single_root_node() {
        let node = ExecutionNode {
            uuid: Some("root-1".to_string()),
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
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.stats.total_nodes, 1);
        assert_eq!(tree.roots[0].depth, 0);
    }

    #[test]
    fn test_parent_child_relationship() {
        let parent = ExecutionNode {
            uuid: Some("parent".to_string()),
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

        let child = ExecutionNode {
            uuid: Some("child".to_string()),
            parent_uuid: Some("parent".to_string()),
            timestamp: Some(2000),
            node_type: "assistant".to_string(),
            message: None,
            tool_use: None,
            tool_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: HashMap::new(),
        };

        let tree = ExecutionTree::from_nodes(vec![parent, child]);
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].children.len(), 1);
        assert_eq!(tree.roots[0].depth, 0);
        assert_eq!(tree.roots[0].children[0].depth, 1);
    }
}
