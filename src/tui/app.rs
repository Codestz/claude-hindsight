//! TUI application state
//!
//! Manages the state of the interactive terminal UI.

use crate::analyzer::{ExecutionTree, TreeNode};
use crate::error::Result;
use crate::parser::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

/// Application state
pub struct App {
    /// The session being viewed
    pub session: Session,

    /// The execution tree
    pub tree: ExecutionTree,

    /// Tree widget state
    pub tree_state: tui_tree_widget::TreeState<String>,

    /// Tree items for rendering
    pub tree_items: Vec<TreeItem<'static, String>>,

    /// Currently selected node index (in DFS order)
    pub selected_index: usize,

    /// All nodes in DFS order
    pub nodes: Vec<TreeNode>,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Whether to quit the application
    pub should_quit: bool,

    /// Status message
    pub status_message: String,
}

impl App {
    /// Create a new app from a session
    pub fn new(session: Session) -> Self {
        let tree = ExecutionTree::from_nodes(session.nodes.clone());
        let nodes = tree
            .depth_first_traversal()
            .into_iter()
            .cloned()
            .collect();

        // Build tree items for tui-tree-widget
        let tree_items = build_tree_items(&tree);

        let mut tree_state = tui_tree_widget::TreeState::default();
        tree_state.select_first();

        App {
            session,
            tree,
            tree_state,
            tree_items,
            selected_index: 0,
            nodes,
            view_mode: ViewMode::Summary,
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

            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.tree_state.key_down();
                self.update_selected_index();
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.tree_state.key_up();
                self.update_selected_index();
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

    /// Update the selected index based on tree state
    fn update_selected_index(&mut self) {
        let selected = self.tree_state.selected();
        // Find the node index by matching the identifier
        if let Some(uuid_str) = selected.first() {
            if let Some(index) = self.nodes.iter().position(|node| {
                node.node.uuid.as_ref().map(|s| s.as_str()) == Some(uuid_str.as_str())
            }) {
                self.selected_index = index;
            }
        }
    }

    /// Get the currently selected node
    pub fn selected_node(&self) -> Option<&TreeNode> {
        self.nodes.get(self.selected_index)
    }
}

/// Build tree items for tui-tree-widget
fn build_tree_items(tree: &ExecutionTree) -> Vec<TreeItem<'static, String>> {
    tree.roots
        .iter()
        .map(|root| build_tree_item(root))
        .collect()
}

/// Recursively build a tree item
fn build_tree_item(node: &TreeNode) -> TreeItem<'static, String> {
    let identifier = node
        .node
        .uuid
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let text = node.summary();

    let children: Vec<TreeItem<String>> = node
        .children
        .iter()
        .map(|child| build_tree_item(child))
        .collect();

    TreeItem::new(identifier, text, children).expect("Failed to create tree item")
}
