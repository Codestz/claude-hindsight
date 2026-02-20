//! Search filter types
#![allow(dead_code)]

/// Search filters for querying sessions
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Filter by tool names (e.g., ["Read", "Write"])
    pub tools: Vec<String>,

    /// Show only sessions with errors
    pub errors_only: bool,

    /// Filter by project name
    pub project: Option<String>,
}

impl SearchFilters {
    /// Create new empty filters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tool filters
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    /// Set errors only filter
    pub fn with_errors_only(mut self, errors_only: bool) -> Self {
        self.errors_only = errors_only;
        self
    }

    /// Set project filter
    pub fn with_project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }

    /// Check if any filters are active
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty() && !self.errors_only && self.project.is_none()
    }
}
