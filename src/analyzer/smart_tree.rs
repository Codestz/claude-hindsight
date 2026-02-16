//! Smart tree builder
//!
//! Creates logical conversation-based tree structure from flat JSONL nodes.

use crate::analyzer::TreeNode;
use crate::parser::ExecutionNode;
use std::collections::HashMap;

/// Build a conversation-based tree from flat nodes
///
/// Groups nodes by user messages and organizes responses, tools, and operations.
pub fn build_conversation_tree(nodes: Vec<ExecutionNode>) -> Vec<TreeNode> {
    let mut conversation_groups: Vec<ConversationGroup> = Vec::new();
    let mut current_group: Option<ConversationGroup> = None;

    for node in nodes {
        match node.node_type.as_str() {
            "user" => {
                // Start new conversation group
                if let Some(group) = current_group.take() {
                    conversation_groups.push(group);
                }
                current_group = Some(ConversationGroup::new(node));
            }
            _ => {
                // Add to current group (or create orphan group if no user message yet)
                if let Some(ref mut group) = current_group {
                    group.add_node(node);
                } else {
                    // Orphan node before first user message - create group
                    let mut group = ConversationGroup::orphan();
                    group.add_node(node);
                    current_group = Some(group);
                }
            }
        }
    }

    // Don't forget the last group
    if let Some(group) = current_group {
        conversation_groups.push(group);
    }

    // Convert groups to TreeNode hierarchy
    conversation_groups
        .into_iter()
        .enumerate()
        .map(|(i, group)| group.to_tree_node(i))
        .collect()
}

/// A group of nodes belonging to one conversation turn
struct ConversationGroup {
    user_message: Option<ExecutionNode>,
    thinking_blocks: Vec<ExecutionNode>,
    tool_calls: Vec<ExecutionNode>,
    tool_results: Vec<ExecutionNode>,
    assistant_messages: Vec<ExecutionNode>,
    file_snapshots: Vec<ExecutionNode>,
    system_messages: Vec<ExecutionNode>,
    progress_updates: Vec<ExecutionNode>,
    other_nodes: Vec<ExecutionNode>,
}

impl ConversationGroup {
    fn new(user_message: ExecutionNode) -> Self {
        ConversationGroup {
            user_message: Some(user_message),
            thinking_blocks: Vec::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            assistant_messages: Vec::new(),
            file_snapshots: Vec::new(),
            system_messages: Vec::new(),
            progress_updates: Vec::new(),
            other_nodes: Vec::new(),
        }
    }

    fn orphan() -> Self {
        ConversationGroup {
            user_message: None,
            thinking_blocks: Vec::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            assistant_messages: Vec::new(),
            file_snapshots: Vec::new(),
            system_messages: Vec::new(),
            progress_updates: Vec::new(),
            other_nodes: Vec::new(),
        }
    }

    fn add_node(&mut self, node: ExecutionNode) {
        match node.node_type.as_str() {
            "thinking" => self.thinking_blocks.push(node),
            "tool_use" => self.tool_calls.push(node),
            "tool_result" => self.tool_results.push(node),
            "assistant" => self.assistant_messages.push(node),
            "file-history-snapshot" => self.file_snapshots.push(node),
            "system" => self.system_messages.push(node),
            "progress" => self.progress_updates.push(node),
            _ => self.other_nodes.push(node),
        }
    }

    fn to_tree_node(self, index: usize) -> TreeNode {
        let mut children = Vec::new();

        // Add thinking blocks
        for node in &self.thinking_blocks {
            children.push(TreeNode {
                node: node.clone(),
                children: Vec::new(),
                depth: 1,
            });
        }

        // Group tool calls with their results
        let tool_children = self.build_tool_tree();
        children.extend(tool_children);

        // Add file snapshots
        if !self.file_snapshots.is_empty() {
            let file_group_node = create_group_node(
                format!("file-ops-{}", index),
                " File Operations".to_string(),
                1,
            );
            let file_children: Vec<TreeNode> = self
                .file_snapshots
                .into_iter()
                .map(|node| TreeNode {
                    node,
                    children: Vec::new(),
                    depth: 2,
                })
                .collect();
            children.push(TreeNode {
                node: file_group_node,
                children: file_children,
                depth: 1,
            });
        }

        // Add assistant messages
        for node in self.assistant_messages {
            children.push(TreeNode {
                node,
                children: Vec::new(),
                depth: 1,
            });
        }

        // Add system messages
        for node in self.system_messages {
            children.push(TreeNode {
                node,
                children: Vec::new(),
                depth: 1,
            });
        }

        // Add progress (collapsed by default, too noisy)
        if !self.progress_updates.is_empty() {
            let progress_group_node = create_group_node(
                format!("progress-{}", index),
                format!(" Progress ({} updates)", self.progress_updates.len()),
                1,
            );
            let progress_children: Vec<TreeNode> = self
                .progress_updates
                .into_iter()
                .map(|node| TreeNode {
                    node,
                    children: Vec::new(),
                    depth: 2,
                })
                .collect();
            children.push(TreeNode {
                node: progress_group_node,
                children: progress_children,
                depth: 1,
            });
        }

        // Add other nodes
        for node in self.other_nodes {
            children.push(TreeNode {
                node,
                children: Vec::new(),
                depth: 1,
            });
        }

        // Create root node (user message or orphan group)
        let root_node = if let Some(user_msg) = self.user_message {
            user_msg
        } else {
            create_group_node(
                format!("orphan-{}", index),
                " Session Start".to_string(),
                0,
            )
        };

        TreeNode {
            node: root_node,
            children,
            depth: 0,
        }
    }

    fn build_tool_tree(&self) -> Vec<TreeNode> {
        if self.tool_calls.is_empty() {
            return Vec::new();
        }

        // Group tools by type
        let mut tools_by_type: HashMap<String, Vec<&ExecutionNode>> = HashMap::new();
        for tool in &self.tool_calls {
            if let Some(ref tool_use) = tool.tool_use {
                tools_by_type
                    .entry(tool_use.name.clone())
                    .or_insert_with(Vec::new)
                    .push(tool);
            }
        }

        // Create tool groups
        let mut tool_children = Vec::new();
        for (tool_name, tools) in tools_by_type {
            let group_label = if tools.len() == 1 {
                format!(" {}", tool_name)
            } else {
                format!(" {} ({}×)", tool_name, tools.len())
            };

            let group_node = create_group_node(
                format!("tool-group-{}", tool_name),
                group_label,
                1,
            );

            let children: Vec<TreeNode> = tools
                .iter()
                .map(|&node| TreeNode {
                    node: node.clone(),
                    children: Vec::new(),
                    depth: 2,
                })
                .collect();

            tool_children.push(TreeNode {
                node: group_node,
                children,
                depth: 1,
            });
        }

        tool_children
    }
}

/// Create a synthetic group node for organizing
fn create_group_node(uuid: String, label: String, depth: usize) -> ExecutionNode {
    ExecutionNode {
        uuid: Some(uuid),
        parent_uuid: None,
        timestamp: None,
        node_type: "group".to_string(),
        message: Some(crate::parser::models::Message {
            role: Some("group".to_string()),
            content: Some(serde_json::json!(label)),
            extra: HashMap::new(),
        }),
        tool_use: None,
        tool_result: None,
        thinking: None,
        progress: None,
        token_usage: None,
        extra: HashMap::new(),
    }
}
