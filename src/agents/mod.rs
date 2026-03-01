//! Agent & Skill discovery and parsing
//!
//! Scans global, per-project, and plugin directories for agent `.md` files
//! and skill `SKILL.md` files. Parses YAML frontmatter + markdown body.

pub mod discovery;
pub mod parser;

pub use discovery::{discover_agents, discover_skills};
pub use parser::{AgentConfig, SkillConfig};
