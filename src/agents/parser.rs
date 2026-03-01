//! Frontmatter parser for agent and skill markdown files.
//!
//! Splits on `---` delimiters, parses YAML frontmatter with serde_yaml,
//! and captures the remaining markdown as the body.

use serde::{Deserialize, Serialize};

/// A supplementary reference or rule file associated with a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReference {
    pub name: String,
    pub path: String,
    pub content: String,
    pub category: String, // "reference" or "rule"
}

/// Parsed agent configuration from an `.md` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolation: Option<String>,

    /// Markdown body (everything after the frontmatter)
    #[serde(skip_deserializing)]
    pub body: String,

    /// Absolute path to the source file
    #[serde(skip_deserializing)]
    pub file_path: String,

    /// Scope: "global", project name, or plugin name
    #[serde(skip_deserializing)]
    pub scope: String,

    /// Project name (if discovered from a project directory)
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
}

/// Parsed skill configuration from a `SKILL.md` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,

    /// Markdown body (everything after the frontmatter)
    #[serde(skip_deserializing)]
    pub body: String,

    /// Absolute path to the source file
    #[serde(skip_deserializing)]
    pub file_path: String,

    /// Scope: "global", project name, or plugin name
    #[serde(skip_deserializing)]
    pub scope: String,

    /// Project name (if discovered from a project directory)
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,

    /// Supplementary reference and rule files from references/ and rules/ subdirs
    #[serde(skip_deserializing, default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<SkillReference>,
}

/// Split a markdown file into (frontmatter_yaml, body).
/// Expects the file to start with `---\n`, followed by YAML, then `---\n`, then body.
/// If no frontmatter is found, returns (None, full_content).
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    // Find the end of the opening ---
    let after_first = match trimmed.strip_prefix("---") {
        Some(rest) => rest.trim_start_matches(['\r', '\n']),
        None => return (None, content),
    };

    // Find the closing ---
    if let Some(end_idx) = after_first.find("\n---") {
        let yaml = &after_first[..end_idx];
        let body_start = end_idx + 4; // skip \n---
        let body = after_first[body_start..].trim_start_matches(['\r', '\n']);
        (Some(yaml), body)
    } else {
        (None, content)
    }
}

/// Parse an agent markdown file into an AgentConfig.
pub fn parse_agent(content: &str, file_path: &str, scope: &str, project_name: Option<&str>) -> Option<AgentConfig> {
    let (yaml, body) = split_frontmatter(content);

    let mut config: AgentConfig = match yaml {
        Some(y) => serde_yaml::from_str(y).ok()?,
        None => {
            // No frontmatter — derive name from filename
            let name = std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            AgentConfig {
                name,
                description: None,
                tools: None,
                model: None,
                permission_mode: None,
                max_turns: None,
                skills: None,
                hooks: None,
                memory: None,
                background: None,
                isolation: None,
                body: String::new(),
                file_path: String::new(),
                scope: String::new(),
                project_name: None,
            }
        }
    };

    config.body = body.to_string();
    config.file_path = file_path.to_string();
    config.scope = scope.to_string();
    config.project_name = project_name.map(|s| s.to_string());

    Some(config)
}

/// Parse a skill SKILL.md file into a SkillConfig.
pub fn parse_skill(content: &str, file_path: &str, scope: &str, project_name: Option<&str>) -> Option<SkillConfig> {
    let (yaml, body) = split_frontmatter(content);

    let mut config: SkillConfig = match yaml {
        Some(y) => serde_yaml::from_str(y).ok()?,
        None => {
            // Derive name from parent directory
            let name = std::path::Path::new(file_path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            SkillConfig {
                name,
                description: None,
                user_invocable: None,
                allowed_tools: None,
                model: None,
                context: None,
                agent: None,
                hooks: None,
                body: String::new(),
                file_path: String::new(),
                scope: String::new(),
                project_name: None,
                references: Vec::new(),
            }
        }
    };

    config.body = body.to_string();
    config.file_path = file_path.to_string();
    config.scope = scope.to_string();
    config.project_name = project_name.map(|s| s.to_string());

    Some(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = "---\nname: test-agent\ndescription: A test\n---\n\n# System prompt\nDo things.";
        let (yaml, body) = split_frontmatter(content);
        assert!(yaml.is_some());
        assert!(yaml.unwrap().contains("name: test-agent"));
        assert!(body.contains("# System prompt"));
    }

    #[test]
    fn test_split_no_frontmatter() {
        let content = "# Just markdown\nNo frontmatter here.";
        let (yaml, body) = split_frontmatter(content);
        assert!(yaml.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_agent() {
        let content = "---\nname: code-reviewer\ndescription: Reviews code\nmodel: claude-sonnet-4-6\n---\nYou are a code reviewer.";
        let agent = parse_agent(content, "/path/to/agent.md", "global", None).unwrap();
        assert_eq!(agent.name, "code-reviewer");
        assert_eq!(agent.description.as_deref(), Some("Reviews code"));
        assert_eq!(agent.model.as_deref(), Some("claude-sonnet-4-6"));
        assert!(agent.body.contains("code reviewer"));
    }

    #[test]
    fn test_parse_skill() {
        let content = "---\nname: commit\ndescription: Create git commits\nuser_invocable: true\n---\nCommit instructions.";
        let skill = parse_skill(content, "/path/to/commit/SKILL.md", "global", None).unwrap();
        assert_eq!(skill.name, "commit");
        assert_eq!(skill.user_invocable, Some(true));
    }
}
