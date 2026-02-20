//! Configuration management for Hindsight
//!
//! Handles loading, saving, and validating user configuration from ~/.config/hindsight/config.toml

use crate::error::{HindsightError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// UI preferences
    #[serde(default)]
    pub ui: UiConfig,

    /// Key bindings
    #[serde(default)]
    pub keybindings: KeyBindings,

    /// Analytics preferences
    #[serde(default)]
    pub analytics: AnalyticsConfig,

    /// Performance settings
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Path settings
    #[serde(default)]
    pub paths: PathsConfig,
}

/// A single configured Claude session directory with optional display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeDirConfig {
    /// Directory path (absolute or `~`-prefixed)
    pub path: String,

    /// Optional human-readable name (e.g., "Work", "Personal")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ClaudeDirConfig {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.path)
    }
}

/// Used for backward-compatible deserialization: old configs use plain strings.
#[derive(Deserialize)]
#[serde(untagged)]
enum ClaudeDirEntry {
    Simple(String),
    Full(ClaudeDirConfig),
}

fn deserialize_claude_dirs<'de, D>(deserializer: D) -> std::result::Result<Vec<ClaudeDirConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let entries = Vec::<ClaudeDirEntry>::deserialize(deserializer)?;
    Ok(entries
        .into_iter()
        .map(|e| match e {
            ClaudeDirEntry::Simple(s) => ClaudeDirConfig { path: s, name: None },
            ClaudeDirEntry::Full(c) => c,
        })
        .collect())
}

/// Paths configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Directories to scan for Claude Code session files.
    /// Each entry can be a plain path string (old format) or a table with `path` + optional `name`.
    /// Default: [{ path = "~/.claude/projects" }]
    #[serde(default = "default_claude_dirs", deserialize_with = "deserialize_claude_dirs")]
    pub claude_dirs: Vec<ClaudeDirConfig>,
}

impl Default for PathsConfig {
    fn default() -> Self {
        PathsConfig {
            claude_dirs: default_claude_dirs(),
        }
    }
}

fn default_claude_dirs() -> Vec<ClaudeDirConfig> {
    vec![ClaudeDirConfig {
        path: "~/.claude/projects".to_string(),
        name: None,
    }]
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme (default, dracula, nord, gruvbox, monokai)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Show analytics panel
    #[serde(default = "default_true")]
    pub show_analytics: bool,

    /// Default view on startup (projects, last_session)
    #[serde(default = "default_view")]
    pub default_view: String,

    /// Show line numbers in details pane
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
}

/// Key bindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_quit")]
    pub quit: String,

    #[serde(default = "default_refresh")]
    pub refresh: String,

    #[serde(default = "default_search")]
    pub search: String,

    #[serde(default = "default_settings")]
    pub settings: String,

    #[serde(default = "default_back")]
    pub back: String,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Show top tools section
    #[serde(default = "default_true")]
    pub show_top_tools: bool,

    /// Show subagent count
    #[serde(default = "default_true")]
    pub show_subagent_count: bool,

    /// Number of top tools to display
    #[serde(default = "default_tools_limit")]
    pub tools_limit: usize,

    /// Show this week/today activity
    #[serde(default = "default_true")]
    pub show_activity: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Cache parsed sessions in memory
    #[serde(default = "default_true")]
    pub cache_sessions: bool,

    /// Maximum cache size in MB
    #[serde(default = "default_cache_size")]
    pub max_cache_size_mb: usize,

    /// Maximum sessions to analyze for top tools
    #[serde(default = "default_max_sessions")]
    pub max_sessions_for_tools: usize,
}

// Default value functions
fn default_theme() -> String {
    "default".to_string()
}

fn default_view() -> String {
    "projects".to_string()
}

fn default_quit() -> String {
    "q".to_string()
}

fn default_refresh() -> String {
    "r".to_string()
}

fn default_search() -> String {
    "/".to_string()
}

fn default_settings() -> String {
    "s".to_string()
}

fn default_back() -> String {
    "h".to_string()
}

fn default_true() -> bool {
    true
}

fn default_tools_limit() -> usize {
    5
}

fn default_cache_size() -> usize {
    100
}

fn default_max_sessions() -> usize {
    100
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            theme: default_theme(),
            show_analytics: default_true(),
            default_view: default_view(),
            show_line_numbers: default_true(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings {
            quit: default_quit(),
            refresh: default_refresh(),
            search: default_search(),
            settings: default_settings(),
            back: default_back(),
        }
    }
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        AnalyticsConfig {
            show_top_tools: default_true(),
            show_subagent_count: default_true(),
            tools_limit: default_tools_limit(),
            show_activity: default_true(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig {
            cache_sessions: default_true(),
            max_cache_size_mb: default_cache_size(),
            max_sessions_for_tools: default_max_sessions(),
        }
    }
}

impl Config {
    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            HindsightError::Config("Could not determine config directory".to_string())
        })?;

        let hindsight_dir = config_dir.join("claude-hindsight");
        Ok(hindsight_dir.join("config.toml"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)
                .map_err(|e| HindsightError::Config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| HindsightError::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, toml_string)?;
        Ok(())
    }

    /// Reset to default configuration
    pub fn reset() -> Result<Self> {
        let config = Config::default();
        config.save()?;
        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate theme
        let valid_themes = ["default", "dracula", "nord", "gruvbox", "monokai"];
        if !valid_themes.contains(&self.ui.theme.as_str()) {
            return Err(HindsightError::Config(format!(
                "Invalid theme '{}'. Valid themes: {}",
                self.ui.theme,
                valid_themes.join(", ")
            )));
        }

        // Validate default view
        let valid_views = ["projects", "last_session"];
        if !valid_views.contains(&self.ui.default_view.as_str()) {
            return Err(HindsightError::Config(format!(
                "Invalid default_view '{}'. Valid views: {}",
                self.ui.default_view,
                valid_views.join(", ")
            )));
        }

        // Validate tools limit
        if self.analytics.tools_limit == 0 || self.analytics.tools_limit > 20 {
            return Err(HindsightError::Config(
                "tools_limit must be between 1 and 20".to_string(),
            ));
        }

        // Validate cache size
        if self.performance.max_cache_size_mb > 1000 {
            return Err(HindsightError::Config(
                "max_cache_size_mb cannot exceed 1000 MB".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ui.theme, "default");
        assert_eq!(config.keybindings.quit, "q");
        assert_eq!(config.analytics.tools_limit, 5);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Test invalid theme
        config.ui.theme = "invalid".to_string();
        assert!(config.validate().is_err());

        // Reset and test invalid tools limit
        config = Config::default();
        config.analytics.tools_limit = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let toml_string = toml::to_string_pretty(&config).unwrap();
        assert!(toml_string.contains("[ui]"));
        assert!(toml_string.contains("[keybindings]"));
        assert!(toml_string.contains("[analytics]"));
        assert!(toml_string.contains("[performance]"));
    }
}
