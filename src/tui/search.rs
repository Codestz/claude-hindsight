//! Search and filter functionality for TUI
//!
//! Filters nodes by type (e.g., user, assistant, tool_use).

use crate::analyzer::TreeNode;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub node_types: HashSet<String>, // Set of node types to filter by
    pub matches: Vec<String>,        // UUIDs of matching nodes
    pub current_match: usize,
}

impl SearchState {
    pub fn new(query: String) -> Self {
        SearchState {
            query,
            node_types: HashSet::new(),
            matches: vec![],
            current_match: 0,
        }
    }

    /// Parse query into node types (comma-separated)
    pub fn parse_query(&mut self) {
        self.node_types.clear();
        for node_type in self.query.split(',') {
            let trimmed = node_type.trim().to_lowercase();
            if !trimmed.is_empty() {
                self.node_types.insert(trimmed);
            }
        }
    }

    /// Check if a node matches any of the filtered node types
    pub fn matches_node(&self, node: &TreeNode) -> bool {
        if self.node_types.is_empty() {
            return true; // No filter active, show all
        }
        self.node_types
            .contains(&node.node.node_type.to_lowercase())
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
