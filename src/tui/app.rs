//! TUI application state
//!
//! Manages the state of the interactive terminal UI.

use crate::analyzer::{build_simple_tree, TreeNode};
use crate::error::Result;
use crate::parser::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use tui_tree_widget::TreeItem;

/// View mode for the details panel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// Show node summary
    Summary,
    /// Show tool input (if available)
    Input,
    /// Show tool output (if available)
    Output,
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

    /// Currently selected node index (in DFS order)
    pub selected_index: usize,

    /// All nodes in DFS order
    pub nodes: Vec<TreeNode>,

    /// Mapping from UUID to node (for fast lookup)
    pub uuid_to_node: HashMap<String, TreeNode>,

    /// Total node count
    pub total_nodes: usize,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Current focus mode
    pub focus_mode: FocusMode,

    /// Scroll offset for details pane
    pub details_scroll: usize,

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
        let tree_items = build_tree_items(&tree_roots);

        let mut tree_state = tui_tree_widget::TreeState::default();
        tree_state.select_first();

        // Get first node as selected
        let nodes = vec![]; // No longer needed
        let selected_index = 0; // No longer needed

        App {
            session,
            tree_roots,
            tree_state,
            tree_items,
            selected_index,
            nodes,
            uuid_to_node,
            total_nodes,
            view_mode: ViewMode::Summary,
            focus_mode: FocusMode::Tree,
            details_scroll: 0,
            should_quit: false,
            status_message: String::new(),
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Navigation - depends on focus mode
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                match self.focus_mode {
                    FocusMode::Tree => {
                        self.tree_state.key_down();
                        self.update_selected_index();
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
                        self.update_selected_index();
                        self.details_scroll = 0; // Reset scroll when changing nodes
                    }
                    FocusMode::Details => {
                        self.details_scroll = self.details_scroll.saturating_sub(1);
                    }
                }
            }

            // Expand/collapse
            (KeyCode::Enter, KeyModifiers::NONE)
            | (KeyCode::Char(' '), KeyModifiers::NONE)
            | (KeyCode::Right, _) => {
                self.tree_state.toggle_selected();
            }
            (KeyCode::Left, _) => {
                self.tree_state.toggle_selected(); // Toggle to close if open
            }

            // Focus switching
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focus_mode = match self.focus_mode {
                    FocusMode::Tree => {
                        self.status_message = "Focus: Details (use j/k to scroll)".to_string();
                        FocusMode::Details
                    }
                    FocusMode::Details => {
                        self.status_message = "Focus: Tree".to_string();
                        FocusMode::Tree
                    }
                };
            }

            // View modes
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.view_mode = ViewMode::Input;
                self.status_message = "Viewing: Tool Input".to_string();
            }
            (KeyCode::Char('o'), KeyModifiers::NONE) => {
                self.view_mode = ViewMode::Output;
                self.status_message = "Viewing: Tool Output".to_string();
            }
            (KeyCode::Char('s'), KeyModifiers::NONE) | (KeyCode::Esc, _) => {
                self.view_mode = ViewMode::Summary;
                self.status_message = "Viewing: Summary".to_string();
            }

            // Home/End
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.tree_state.select_first();
                self.update_selected_index();
            }
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.tree_state.select_last();
                self.update_selected_index();
            }

            _ => {}
        }

        Ok(())
    }

    /// Update the selected index based on tree state (no longer needed with UUID lookup)
    fn update_selected_index(&mut self) {
        // No-op: we now use UUID-based lookup in selected_node()
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
fn build_tree_items(roots: &[TreeNode]) -> Vec<TreeItem<'static, String>> {
    roots.iter().map(|root| build_tree_item(root)).collect()
}

/// Recursively build a tree item using UUID as identifier
fn build_tree_item(node: &TreeNode) -> TreeItem<'static, String> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

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

    let children: Vec<TreeItem<String>> = node
        .children
        .iter()
        .map(|child| build_tree_item(child))
        .collect();

    TreeItem::new(identifier, styled_line, children).expect("Failed to create tree item")
}
