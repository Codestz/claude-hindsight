//! Search and filter functionality for TUI
//!
//! Filters nodes by type (e.g., user, assistant, tool_use) with semantic aliases.

use crate::analyzer::TreeNode;
use crate::parser::models::ContentBlock;
use std::collections::HashSet;

/// Search mode: type filter (`/`) or keyword search (`?`)
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    /// Filter by node type (the `/` command)
    TypeFilter,
    /// Search by keyword in content (the `?` command)
    KeywordSearch,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub node_types: HashSet<String>, // Set of node types to filter by
    pub matches: Vec<String>,        // UUIDs of matching nodes
    pub current_match: usize,
    pub mode: SearchMode,
}

impl SearchState {
    pub fn new(query: String) -> Self {
        SearchState {
            query,
            node_types: HashSet::new(),
            matches: vec![],
            current_match: 0,
            mode: SearchMode::TypeFilter,
        }
    }

    pub fn new_keyword(query: String) -> Self {
        SearchState {
            query,
            node_types: HashSet::new(),
            matches: vec![],
            current_match: 0,
            mode: SearchMode::KeywordSearch,
        }
    }

    /// Parse query into node types (comma-separated) with semantic alias expansion
    pub fn parse_query(&mut self) {
        self.node_types.clear();
        for node_type in self.query.split(',') {
            let trimmed = node_type.trim().to_lowercase();
            if !trimmed.is_empty() {
                self.node_types.insert(trimmed);
            }
        }
    }

    /// Check if a node matches the current filter.
    ///
    /// For TypeFilter mode: matches node types with semantic aliases.
    /// For KeywordSearch mode: searches node content by keyword.
    pub fn matches_node(&self, node: &TreeNode) -> bool {
        match self.mode {
            SearchMode::TypeFilter => self.matches_type_filter(node),
            SearchMode::KeywordSearch => self.matches_keyword(node),
        }
    }

    /// Type-based filtering with semantic aliases:
    /// - "error" → matches node_type "error" OR tool_result.is_error OR <tool_use_error>
    /// - "prompt" → matches user messages with prompt_score >= 40%
    /// - "tool" → matches both "tool_use" and "tool_result" (assistant tool calls + user tool results)
    /// - Anything else → literal node_type match (existing behavior)
    fn matches_type_filter(&self, node: &TreeNode) -> bool {
        if self.node_types.is_empty() {
            return true;
        }

        let nt = node.node.node_type;

        for filter in &self.node_types {
            match filter.as_str() {
                "error" => {
                    if node.node.has_error() {
                        return true;
                    }
                }
                "tool" => {
                    // Also match assistant nodes that contain ToolUse blocks
                    if nt == crate::parser::models::NodeType::Assistant {
                        if let Some(ref msg) = node.node.message {
                            if msg
                                .content_blocks()
                                .iter()
                                .any(|b| matches!(b, ContentBlock::ToolUse { .. }))
                            {
                                return true;
                            }
                        }
                    }
                    // Also match user nodes that contain ToolResult blocks
                    if nt == crate::parser::models::NodeType::User {
                        if let Some(ref msg) = node.node.message {
                            if msg
                                .content_blocks()
                                .iter()
                                .any(|b| matches!(b, ContentBlock::ToolResult { .. }))
                            {
                                return true;
                            }
                        }
                    }
                }
                "prompt" => {
                    // Match user messages with prompt_score >= 40
                    // We can't compute full context here, so use a simplified check:
                    // user node with meaningful text content
                    if nt == crate::parser::models::NodeType::User {
                        if let Some(ref msg) = node.node.message {
                            // Skip tool-result-only messages
                            let blocks = msg.content_blocks();
                            if !blocks.is_empty() {
                                let has_text = blocks.iter().any(|b| {
                                    matches!(b, ContentBlock::Text { text } if !text.trim().is_empty())
                                });
                                let has_tool_result = blocks
                                    .iter()
                                    .any(|b| matches!(b, ContentBlock::ToolResult { .. }));
                                if has_tool_result && !has_text {
                                    continue;
                                }
                            }
                            let text = msg.text_content();
                            let text = text.trim();
                            // Require text length >= 10 chars (filters "ok", "yes", "y")
                            if text.chars().count() >= 10 {
                                return true;
                            }
                        }
                    }
                }
                _ => {
                    // Literal match (existing behavior)
                    if self.node_types.contains(&nt.as_str().to_string()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Keyword-based content search.
    /// Searches: message text, tool names, tool inputs, thinking, tool results.
    fn matches_keyword(&self, node: &TreeNode) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let query_lower = self.query.to_lowercase();

        // Search in node_type
        if node.node.node_type.as_str().contains(&query_lower) {
            return true;
        }

        // Search in tool_use name
        if let Some(ref tool_use) = node.node.tool_use {
            if tool_use.name.to_lowercase().contains(&query_lower) {
                return true;
            }
            // Search tool input as JSON string
            let input_str = tool_use.input.to_string().to_lowercase();
            if input_str.contains(&query_lower) {
                return true;
            }
        }

        // Search in message content
        if let Some(ref message) = node.node.message {
            let text = message.text_content();
            if text.to_lowercase().contains(&query_lower) {
                return true;
            }
            // Also search tool names in typed blocks
            for block in message.content_blocks() {
                match block {
                    ContentBlock::ToolUse { name, input, .. } => {
                        if name.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                        if input.to_string().to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                    ContentBlock::ToolResult { content: Some(v), .. } => {
                        let s = if let Some(s) = v.as_str() {
                            s.to_string()
                        } else {
                            v.to_string()
                        };
                        if s.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                    ContentBlock::ToolResult { content: None, .. } => {}
                    _ => {}
                }
            }
        }

        // Search in thinking content
        if let Some(ref thinking) = node.node.thinking {
            if thinking.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        // Search in tool result content
        if let Some(ref tool_result) = node.node.tool_result {
            if let Some(ref content) = tool_result.content {
                if content.to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
            if let Some(ref error) = tool_result.error {
                if error.to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
        }

        false
    }

    /// Move to next match
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    /// Move to previous match
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.matches.len() - 1
            } else {
                self.current_match - 1
            };
        }
    }

    /// Get UUID of current match
    pub fn current_match_uuid(&self) -> Option<&str> {
        self.matches.get(self.current_match).map(|s| s.as_str())
    }
}
