//! Agent & Skill discovery across global, per-project, and plugin directories.
//!
//! Scans:
//! 1. Global:      `~/.claude/agents/*.md`, `~/.claude/skills/*/SKILL.md`
//! 2. Per-project:  For each encoded project dir in `claude_dirs`, decode back to real path,
//!                  then scan `<real_path>/.claude/agents/` and `<real_path>/.claude/skills/`
//! 3. Plugins:     `~/.claude/plugins/**/agents/*.md`, `~/.claude/plugins/**/skills/*/SKILL.md`

use super::parser::{self, AgentConfig, SkillConfig, SkillReference};
use std::fs;
use std::path::{Path, PathBuf};

fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

fn claude_home() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".claude"))
}

/// Decode an encoded directory name back to a real filesystem path.
///
/// Claude Code encodes paths by replacing `/` with `-`, so
/// `-Users-codestz-Documents-PersonalProjects-claude-hindsight`
/// means `/Users/codestz/Documents/PersonalProjects/claude-hindsight`.
///
/// Since directory names can contain literal dashes (e.g. `claude-hindsight`),
/// we can't just replace all dashes with `/`. Instead we greedily walk the
/// filesystem: at each segment boundary (dash) we check if extending the
/// current segment (keeping the dash literal) matches an existing directory,
/// or if starting a new path component matches. We prefer the longest existing
/// path.
fn decode_dir_name(encoded: &str) -> PathBuf {
    let stripped = match encoded.strip_prefix('-') {
        Some(s) => s,
        None => return PathBuf::from(encoded),
    };

    let parts: Vec<&str> = stripped.split('-').collect();
    if parts.is_empty() {
        return PathBuf::from(encoded);
    }

    // Greedy: build path by trying to join dashes into the current segment
    // when the slash interpretation doesn't lead to existing dirs.
    fn solve(parts: &[&str], idx: usize, current: &Path) -> Option<PathBuf> {
        if idx >= parts.len() {
            return if current.exists() { Some(current.to_path_buf()) } else { None };
        }

        // Try accumulating segments with dashes (literal dash in dir name)
        // from longest to shortest
        for end in (idx + 1..=parts.len()).rev() {
            let segment = parts[idx..end].join("-");
            let candidate = current.join(&segment);
            if candidate.exists() {
                if end == parts.len() {
                    return Some(candidate);
                }
                if let Some(result) = solve(parts, end, &candidate) {
                    return Some(result);
                }
            }
        }

        // Single segment as path component (may not exist yet — last resort)
        let candidate = current.join(parts[idx]);
        solve(parts, idx + 1, &candidate)
    }

    let root = PathBuf::from("/");
    solve(&parts, 0, &root).unwrap_or_else(|| {
        // Fallback: naive replacement (best effort)
        PathBuf::from(format!("/{}", stripped.replace('-', "/")))
    })
}

/// Scan a directory for `*.md` files and parse each as an agent.
fn scan_agents_dir(dir: &Path, scope: &str, project_name: Option<&str>) -> Vec<AgentConfig> {
    let mut agents = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return agents,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(agent) = parser::parse_agent(
                    &content,
                    &path.to_string_lossy(),
                    scope,
                    project_name,
                ) {
                    agents.push(agent);
                }
            }
        }
    }
    agents
}

/// Scan a skill directory for `references/` and `rules/` subdirectories containing `.md` files.
fn scan_skill_references(skill_dir: &Path) -> Vec<SkillReference> {
    let mut refs = Vec::new();
    for (subdir, category) in &[("references", "reference"), ("rules", "rule")] {
        let dir = skill_dir.join(subdir);
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            refs.push(SkillReference {
                                name,
                                path: path.to_string_lossy().to_string(),
                                content,
                                category: category.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    refs.sort_by(|a, b| a.name.cmp(&b.name));
    refs
}

/// Scan a directory for `*/SKILL.md` files and parse each as a skill.
fn scan_skills_dir(dir: &Path, scope: &str, project_name: Option<&str>) -> Vec<SkillConfig> {
    let mut skills = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return skills,
    };

    for entry in entries.flatten() {
        let skill_dir = entry.path();
        if skill_dir.is_dir() {
            let skill_file = skill_dir.join("SKILL.md");
            if skill_file.is_file() {
                if let Ok(content) = fs::read_to_string(&skill_file) {
                    if let Some(mut skill) = parser::parse_skill(
                        &content,
                        &skill_file.to_string_lossy(),
                        scope,
                        project_name,
                    ) {
                        skill.references = scan_skill_references(&skill_dir);
                        skills.push(skill);
                    }
                }
            }
        }
    }
    skills
}

/// Discover all agents from global, per-project, and plugin directories.
pub fn discover_agents() -> Vec<AgentConfig> {
    let mut all = Vec::new();

    let Some(claude) = claude_home() else {
        return all;
    };

    // 1. Global agents: ~/.claude/agents/*.md
    let global_agents = claude.join("agents");
    all.extend(scan_agents_dir(&global_agents, "global", None));

    // 2. Per-project agents
    let config = crate::config::Config::load().unwrap_or_default();
    let home = home_dir().unwrap_or_default();

    for dir_cfg in &config.paths.claude_dirs {
        let expanded = if let Some(stripped) = dir_cfg.path.strip_prefix("~/") {
            home.join(stripped)
        } else {
            PathBuf::from(&dir_cfg.path)
        };

        let entries = match fs::read_dir(&expanded) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            let dir_name = project_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            // Decode encoded dir name to real path
            let real_path = decode_dir_name(dir_name);

            // Extract short project name (last segment)
            let project_name = real_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(dir_name);

            // Scan <real_project_path>/.claude/agents/
            let project_agents_dir = real_path.join(".claude").join("agents");
            all.extend(scan_agents_dir(&project_agents_dir, project_name, Some(project_name)));
        }
    }

    // 3. Plugin agents: ~/.claude/plugins/*/agents/*.md
    let plugins_dir = claude.join("plugins");
    if plugins_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let plugin_dir = entry.path();
                if plugin_dir.is_dir() {
                    let plugin_name = plugin_dir
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    let agents_dir = plugin_dir.join("agents");
                    all.extend(scan_agents_dir(&agents_dir, &format!("plugin:{plugin_name}"), None));
                }
            }
        }
    }

    // Sort by name for consistent output
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}

/// Discover all skills from global, per-project, and plugin directories.
pub fn discover_skills() -> Vec<SkillConfig> {
    let mut all = Vec::new();

    let Some(claude) = claude_home() else {
        return all;
    };

    // 1. Global skills: ~/.claude/skills/*/SKILL.md
    let global_skills = claude.join("skills");
    all.extend(scan_skills_dir(&global_skills, "global", None));

    // 2. Per-project skills
    let config = crate::config::Config::load().unwrap_or_default();
    let home = home_dir().unwrap_or_default();

    for dir_cfg in &config.paths.claude_dirs {
        let expanded = if let Some(stripped) = dir_cfg.path.strip_prefix("~/") {
            home.join(stripped)
        } else {
            PathBuf::from(&dir_cfg.path)
        };

        let entries = match fs::read_dir(&expanded) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            let dir_name = project_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let real_path = decode_dir_name(dir_name);
            let project_name = real_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(dir_name);

            let project_skills_dir = real_path.join(".claude").join("skills");
            all.extend(scan_skills_dir(&project_skills_dir, project_name, Some(project_name)));
        }
    }

    // 3. Plugin skills: ~/.claude/plugins/*/skills/*/SKILL.md
    let plugins_dir = claude.join("plugins");
    if plugins_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let plugin_dir = entry.path();
                if plugin_dir.is_dir() {
                    let plugin_name = plugin_dir
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    let skills_dir = plugin_dir.join("skills");
                    all.extend(scan_skills_dir(&skills_dir, &format!("plugin:{plugin_name}"), None));
                }
            }
        }
    }

    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_dir_name_no_prefix() {
        let decoded = decode_dir_name("my-project");
        assert_eq!(decoded, PathBuf::from("my-project"));
    }

    #[test]
    fn test_decode_dir_name_fallback() {
        // When path doesn't exist on disk, falls back to naive replacement
        let decoded = decode_dir_name("-nonexistent-path-here");
        assert_eq!(decoded, PathBuf::from("/nonexistent/path/here"));
    }
}
