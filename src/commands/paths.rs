//! `hindsight paths` — manage Claude project directories to scan

use crate::config::{ClaudeDirConfig, Config};
use crate::error::{HindsightError, Result};

const DEFAULT_DIR: &str = "~/.claude/projects";

/// List all configured scan directories
pub fn list() -> Result<()> {
    let config = Config::load()?;
    let home = dirs::home_dir()
        .ok_or_else(|| HindsightError::Config("Could not determine home directory".to_string()))?;

    println!("  Claude project directories:\n");

    for dir in &config.paths.claude_dirs {
        let expanded = if let Some(rest) = dir.path.strip_prefix("~/") {
            home.join(rest)
        } else {
            std::path::PathBuf::from(&dir.path)
        };

        let exists = expanded.exists();
        let is_default = dir.path == DEFAULT_DIR;

        let status = if exists { "✓" } else { "✗ (not found)" };
        let tag = if is_default { "  [default]" } else { "" };
        let name_tag = dir
            .name
            .as_deref()
            .map(|n| format!("  [{}]", n))
            .unwrap_or_default();

        println!("  {} {}{}{}", status, dir.path, tag, name_tag);
    }

    println!();
    Ok(())
}

/// Add a directory to the scan list
pub fn add(path: String, name: Option<String>) -> Result<()> {
    // Normalise: replace home dir prefix with ~
    let home = dirs::home_dir()
        .ok_or_else(|| HindsightError::Config("Could not determine home directory".to_string()))?;

    let normalised = if let Ok(stripped) = std::path::Path::new(&path).strip_prefix(&home) {
        format!("~/{}", stripped.to_string_lossy())
    } else {
        path.clone()
    };

    let mut config = Config::load()?;

    if config.paths.claude_dirs.iter().any(|d| d.path == normalised) {
        println!("  Already configured: {}", normalised);
        return Ok(());
    }

    config.paths.claude_dirs.push(ClaudeDirConfig {
        path: normalised.clone(),
        name: name.clone(),
    });
    config.save()?;

    if let Some(n) = name {
        println!("  ✓ Added: {} [{}]", normalised, n);
    } else {
        println!("  ✓ Added: {}", normalised);
    }
    println!("  Run `hindsight reindex` to scan the new directory.");
    Ok(())
}

/// Remove a directory from the scan list
pub fn remove(path: String) -> Result<()> {
    // Normalise the same way as add
    let home = dirs::home_dir()
        .ok_or_else(|| HindsightError::Config("Could not determine home directory".to_string()))?;

    let normalised = if let Ok(stripped) = std::path::Path::new(&path).strip_prefix(&home) {
        format!("~/{}", stripped.to_string_lossy())
    } else {
        path.clone()
    };

    if normalised == DEFAULT_DIR {
        eprintln!("  Cannot remove the default directory ({}).", DEFAULT_DIR);
        return Err(HindsightError::Config(
            "The default directory cannot be removed.".to_string(),
        ));
    }

    let mut config = Config::load()?;

    let before = config.paths.claude_dirs.len();
    config.paths.claude_dirs.retain(|d| d.path != normalised);

    if config.paths.claude_dirs.len() == before {
        eprintln!("  Not found in config: {}", normalised);
        return Err(HindsightError::Config(format!(
            "'{}' is not in the configured directories.",
            normalised
        )));
    }

    config.save()?;
    println!("  ✓ Removed: {}", normalised);
    Ok(())
}
