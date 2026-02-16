//! TUI application state
//!
//! Manages the state of the interactive terminal UI.

use crate::analyzer::{build_simple_tree, TreeNode};
use crate::error::Result;
use crate::parser::Session;
use crate::tui::search::SearchState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use tui_tree_widget::TreeItem;

/// Scroll position information for displaying scroll indicators
#[derive(Debug, Clone)]
pub struct ScrollInfo {
    pub offset: usize,
    pub total_lines: usize,
    pub viewport_height: usize,
}

impl ScrollInfo {
    pub fn new() -> Self {
        ScrollInfo {
            offset: 0,
            total_lines: 0,
            viewport_height: 0,
        }
    }

    pub fn position_text(&self) -> String {
        if self.total_lines == 0 {
            return String::new();
        }
        let current = self.offset + 1;
        let end = (self.offset + self.viewport_height).min(self.total_lines);
        format!("Lines {}-{}/{}", current, end, self.total_lines)
    }
}

/// Focus mode - which pane is active
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusMode {
    /// Tree is focused (default)
    Tree,
    /// Details pane is focused
    Details,
}

/// Application state
pub struct App {
    /// The session being viewed
    pub session: Session,

    /// Conversation-based tree (smart grouping)
    pub tree_roots: Vec<TreeNode>,

    /// Tree widget state
    pub tree_state: tui_tree_widget::TreeState<String>,

    /// Tree items for rendering
    pub tree_items: Vec<TreeItem<'static, String>>,

    /// Mapping from UUID to node (for fast lookup)
    pub uuid_to_node: HashMap<String, TreeNode>,

    /// Total node count
    pub total_nodes: usize,

    /// Current focus mode
    pub focus_mode: FocusMode,

    /// Scroll offset for details pane
    pub details_scroll: usize,

    /// Scroll information for details pane
    pub details_scroll_info: ScrollInfo,

    /// Search state (None if not searching)
    pub search_state: Option<SearchState>,

    /// Whether we're in input mode (typing search query)
    pub input_mode: bool,

    /// Whether to quit the application
    pub should_quit: bool,

    /// Status message
    pub status_message: String,
}

impl App {
    /// Create a new app from a session
    pub fn new(session: Session) -> Self {
        let total_nodes = session.nodes.len();

        // Build simple hierarchical tree from parent_uuid relationships
        let tree_roots = build_simple_tree(session.nodes.clone());

        // Build UUID-to-node mapping for fast lookup
        let mut uuid_to_node = HashMap::new();
        for root in &tree_roots {
            collect_uuid_mapping(root, &mut uuid_to_node);
        }

        // Build tree items for tui-tree-widget (using UUIDs as identifiers)
        let tree_items = build_tree_items(&tree_roots, &None);

        let mut tree_state = tui_tree_widget::TreeState::default();
        tree_state.select_first();

        App {
            session,
            tree_roots,
            tree_state,
            tree_items,
            uuid_to_node,
            total_nodes,
            focus_mode: FocusMode::Tree,
            details_scroll: 0,
            details_scroll_info: ScrollInfo::new(),
            search_state: None,
            input_mode: false,
            should_quit: false,
            status_message: String::new(),
        }
    }

    /// Update scroll info (called by UI during rendering)
    pub fn update_scroll_info(&mut self, total_lines: usize, viewport_height: usize) {
        self.details_scroll_info.offset = self.details_scroll;
        self.details_scroll_info.total_lines = total_lines;
        self.details_scroll_info.viewport_height = viewport_height;
    }

    /// Get breadcrumb path for currently selected node
    pub fn get_breadcrumb_path(&self) -> Vec<String> {
        let selected = match self.selected_node() {
            Some(node) => node,
            None => return vec![],
        };

        let mut path = vec![];
        let mut current_uuid = selected.node.uuid.clone();

        while let Some(uuid) = current_uuid {
            if let Some(node) = self.uuid_to_node.get(&uuid) {
                let label = match node.node.node_type.as_str() {
                    "user" => "User".to_string(),
                    "assistant" => "Assistant".to_string(),
                    "tool_use" => {
                        node.node.tool_use.as_ref()
                            .map(|t| format!("Tool:{}", t.name))
                            .unwrap_or_else(|| "Tool".to_string())
                    }
                    "tool_result" => "Result".to_string(),
                    "thinking" => "Think".to_string(),
                    "progress" => "Progress".to_string(),
                    _ => node.node.node_type.clone(),
                };
                path.push(label);
                current_uuid = node.node.parent_uuid.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.input_mode = true;
        self.search_state = Some(SearchState::new(String::new()));
        self.status_message = "Filter by node type (e.g., user,assistant,tool_use): ".to_string();
    }

    /// Execute search with current query
    pub fn execute_search(&mut self) {
        let first_match_uuid = if let Some(ref mut search) = self.search_state {
            // Parse the query into node types
            search.parse_query();
            search.matches.clear();

            // Find all matching nodes
            for (uuid, node) in &self.uuid_to_node {
                if search.matches_node(node) {
                    search.matches.push(uuid.clone());
                }
            }

            search.current_match = 0;

            let filter_types = search.node_types.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            self.status_message = if search.node_types.is_empty() {
                "Filter cleared - showing all nodes".to_string()
            } else {
                format!("Filtered: {} ({} matches)", filter_types, search.matches.len())
            };

            search.current_match_uuid().map(|s| s.to_string())
        } else {
            None
        };

        // Rebuild tree with filtering
        self.rebuild_tree_items();

        // Jump to first match if any
        if let Some(uuid) = first_match_uuid {
            self.tree_state.select(vec![uuid]);
            self.details_scroll = 0;
        }

        self.input_mode = false;
    }

    /// Jump to next search match
    pub fn next_search_match(&mut self) {
        if let Some(ref mut search) = self.search_state {
            search.next_match();
            if let Some(uuid) = search.current_match_uuid() {
                self.tree_state.select(vec![uuid.to_string()]);
                self.details_scroll = 0;
                self.status_message = format!(
                    "Match {}/{}",
                    search.current_match + 1,
                    search.matches.len()
                );
            }
        }
    }

    /// Jump to previous search match
    pub fn prev_search_match(&mut self) {
        if let Some(ref mut search) = self.search_state {
            search.prev_match();
            if let Some(uuid) = search.current_match_uuid() {
                self.tree_state.select(vec![uuid.to_string()]);
                self.details_scroll = 0;
                self.status_message = format!(
                    "Match {}/{}",
                    search.current_match + 1,
                    search.matches.len()
                );
            }
        }
    }

    /// Cancel search
    pub fn cancel_search(&mut self) {
        self.search_state = None;
        self.input_mode = false;
        self.rebuild_tree_items();
        self.status_message = "Search cancelled".to_string();
    }

    /// Rebuild tree items (used when search state changes)
    pub fn rebuild_tree_items(&mut self) {
        self.tree_items = build_tree_items(&self.tree_roots, &self.search_state);
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle input mode separately
        if self.input_mode {
            return self.handle_input_key(key);
        }

        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Start search
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.start_search();
            }

            // Next match
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                if self.search_state.is_some() {
                    self.next_search_match();
                }
            }

            // Previous match
            (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                if self.search_state.is_some() {
                    self.prev_search_match();
                }
            }

            // Clear search (Alt+c)
            (KeyCode::Char('c'), KeyModifiers::ALT) => {
                if self.search_state.is_some() {
                    self.cancel_search();
                }
            }

            // Navigation - depends on focus mode
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                match self.focus_mode {
                    FocusMode::Tree => {
                        self.tree_state.key_down();
                        self.details_scroll = 0; // Reset scroll when changing nodes
                    }
                    FocusMode::Details => {
                        self.details_scroll = self.details_scroll.saturating_add(1);
                    }
                }
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                match self.focus_mode {
                    FocusMode::Tree => {
                        self.tree_state.key_up();
                        self.details_scroll = 0; // Reset scroll when changing nodes
                    }
                    FocusMode::Details => {
                        self.details_scroll = self.details_scroll.saturating_sub(1);
                    }
                }
            }

            // Half-page scroll (Ctrl+d/u)
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_add(15);
                    self.status_message = "↓ Half page".to_string();
                }
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_sub(15);
                    self.status_message = "↑ Half page".to_string();
                }
            }

            // Full-page scroll (Ctrl+f/b or PageDown/PageUp)
            (KeyCode::Char('f'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_add(30);
                    self.status_message = "↓ Full page".to_string();
                }
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_sub(30);
                    self.status_message = "↑ Full page".to_string();
                }
            }

            // Focus switching
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focus_mode = match self.focus_mode {
                    FocusMode::Tree => {
                        self.status_message = "Focus: Details (use j/k to scroll)".to_string();
                        FocusMode::Details
                    }
                    FocusMode::Details => {
                        self.status_message = "Focus: List".to_string();
                        FocusMode::Tree
                    }
                };
            }

            // Home/End
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.tree_state.select_first();
                self.status_message = "↑ Top".to_string();
            }
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.tree_state.select_last();
                self.status_message = "↓ Bottom".to_string();
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle keyboard input in input mode (for search)
    fn handle_input_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                self.execute_search();
            }
            KeyCode::Esc => {
                self.cancel_search();
            }
            KeyCode::Backspace => {
                if let Some(ref mut search) = self.search_state {
                    search.query.pop();
                    self.status_message = format!("Search: {}", search.query);
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut search) = self.search_state {
                    search.query.push(c);
                    self.status_message = format!("Search: {}", search.query);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the currently selected node
    pub fn selected_node(&self) -> Option<&TreeNode> {
        // Get selected UUID from tree state
        let selected = self.tree_state.selected();
        if let Some(uuid) = selected.first() {
            // Look up node by UUID
            self.uuid_to_node.get(uuid).map(|n| n)
        } else {
            None
        }
    }
}

/// Build UUID-to-node mapping recursively
fn collect_uuid_mapping(node: &TreeNode, mapping: &mut HashMap<String, TreeNode>) {
    if let Some(ref uuid) = node.node.uuid {
        mapping.insert(uuid.clone(), node.clone());
    }
    for child in &node.children {
        collect_uuid_mapping(child, mapping);
    }
}

/// Build tree items for tui-tree-widget using UUIDs as identifiers
fn build_tree_items(roots: &[TreeNode], search_state: &Option<SearchState>) -> Vec<TreeItem<'static, String>> {
    roots.iter()
        .filter_map(|root| build_tree_item(root, search_state))
        .collect()
}

/// Recursively build a tree item using UUID as identifier
/// Returns None if the node and all its children are filtered out
fn build_tree_item(node: &TreeNode, search_state: &Option<SearchState>) -> Option<TreeItem<'static, String>> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    // Check if this node matches the filter
    let matches_filter = search_state
        .as_ref()
        .map(|s| s.matches_node(node))
        .unwrap_or(true);

    // Build children first (recursively)
    let children: Vec<TreeItem<String>> = node
        .children
        .iter()
        .filter_map(|child| build_tree_item(child, search_state))
        .collect();

    // If this node doesn't match AND has no matching children, filter it out
    if !matches_filter && children.is_empty() {
        return None;
    }

    // Use UUID as identifier (or generate one for nodes without UUID)
    let identifier = node
        .node
        .uuid
        .clone()
        .unwrap_or_else(|| format!("no-uuid-{}", node.node.node_type));

    // Get smart label with color
    let (label_text, color_name) = crate::analyzer::smart_label::get_node_label(node);

    // Map color name to ratatui Color
    let color = match color_name {
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "gray" => Color::Gray,
        _ => Color::White,
    };

    // Create styled text
    let styled_line = Line::from(Span::styled(label_text, Style::default().fg(color)));

    Some(TreeItem::new(identifier, styled_line, children).expect("Failed to create tree item"))
}
