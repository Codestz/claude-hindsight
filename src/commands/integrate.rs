//! Implementation of the `integrate` command
//!
//! Installs Claude Code lifecycle hooks so Hindsight auto-indexes sessions
//! without any manual `reindex` or startup scan needed.

use crate::error::Result;
use std::io::{self, Write};
use std::path::PathBuf;

/// Legacy hook command (kept for backward-compat detection)
const HOOK_COMMAND_LEGACY: &str = "claude-hindsight hook-index";
/// Bare-name prefix (legacy, pre-absolute-path)
const HOOK_COMMAND_PREFIX: &str = "claude-hindsight hook ";
/// Pattern present in all hindsight hook commands (absolute or bare)
const HOOK_COMMAND_MARKER: &str = "hook ";

/// Resolve the absolute path to the running binary.
/// Falls back to the bare name if current_exe() fails.
fn resolve_binary_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "claude-hindsight".to_string())
}

fn hook_entry(subcommand: &str) -> serde_json::Value {
    let bin = resolve_binary_path();
    serde_json::json!({ "type": "command", "command": format!("{} hook {}", bin, subcommand), "async": true })
}

/// Build the hooks JSON object we want to be present in settings
fn hindsight_hooks() -> serde_json::Value {
    serde_json::json!({
        "hooks": {
            "SessionStart":       [{ "hooks": [hook_entry("session-start")] }],
            "Stop":               [{ "hooks": [hook_entry("stop")] }],
            "SessionEnd":         [{ "hooks": [hook_entry("session-end")] }],
            "PreToolUse":         [{ "hooks": [hook_entry("pre-tool-use")] }],
            "PostToolUse":        [{ "hooks": [hook_entry("post-tool-use")] }],
            "PostToolUseFailure": [{ "hooks": [hook_entry("post-tool-use-failure")] }],
            "SubagentStart":      [{ "hooks": [hook_entry("subagent-start")] }],
            "SubagentStop":       [{ "hooks": [hook_entry("subagent-stop")] }],
            "PreCompact":         [{ "hooks": [hook_entry("pre-compact")] }],
            "PermissionRequest":  [{ "hooks": [hook_entry("permission-request")] }],
            "TaskCompleted":      [{ "hooks": [hook_entry("task-completed")] }],
            "WorktreeCreate":     [{ "hooks": [hook_entry("worktree-create")] }],
            "WorktreeRemove":     [{ "hooks": [hook_entry("worktree-remove")] }],
            "ConfigChange":       [{ "hooks": [hook_entry("config-change")] }],
        }
    })
}

/// Derive settings.json paths from configured claude_dirs.
///
/// Each entry in `paths.claude_dirs` (e.g. `~/.claude/projects`) implies a
/// settings file at `~/.claude/settings.json`.
fn derive_settings_paths() -> Vec<(PathBuf, String)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };

    let config = crate::config::Config::load().unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for entry in &config.paths.claude_dirs {
        // Expand ~ in the path
        let expanded: PathBuf = if let Some(stripped) = entry.path.strip_prefix("~/") {
            home.join(stripped)
        } else {
            PathBuf::from(&entry.path)
        };

        // Parent of the projects dir = Claude Code root (e.g. ~/.claude)
        let parent = match expanded.parent() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        let settings_path = parent.join("settings.json");
        let key = settings_path.to_string_lossy().into_owned();
        if seen.insert(key.clone()) {
            // Human-readable label for the menu
            let label = entry.name.clone().unwrap_or_else(|| {
                // e.g. ~/.claudep/settings.json → ~/.claudep
                parent
                    .to_str()
                    .map(|s| {
                        if let Some(h) = home.to_str() {
                            s.replacen(h, "~", 1)
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or_else(|| key.clone())
            });
            result.push((settings_path, label));
        }
    }

    result
}

/// Check whether any Hindsight hook is already present in a settings value.
/// Matches legacy bare-name commands, absolute-path commands, and the old hook-index form.
fn already_installed(value: &serde_json::Value) -> bool {
    let text = serde_json::to_string(value).unwrap_or_default();
    text.contains(HOOK_COMMAND_LEGACY)
        || text.contains(HOOK_COMMAND_PREFIX)
        || text.contains("claude-hindsight hook ")
        || (text.contains("hindsight") && text.contains(HOOK_COMMAND_MARKER))
}

/// Merge our hooks into an existing JSON value (in place).
///
/// Preserves all existing hooks; appends ours only if not already present.
fn merge_hooks(existing: &mut serde_json::Value) {
    let desired = hindsight_hooks();

    // Ensure top-level "hooks" key exists
    if existing.get("hooks").is_none() {
        existing["hooks"] = serde_json::json!({});
    }

    let hooks_obj = existing["hooks"].as_object_mut().unwrap();

    for (event, desired_entries) in desired["hooks"].as_object().unwrap() {
        let arr = hooks_obj
            .entry(event)
            .or_insert_with(|| serde_json::json!([]));

        if let Some(arr) = arr.as_array_mut() {
            // Only append if not already there (check both bare and absolute paths)
            let text = serde_json::to_string(&arr).unwrap_or_default();
            let has_hindsight = text.contains(HOOK_COMMAND_LEGACY)
                || text.contains(HOOK_COMMAND_PREFIX)
                || (text.contains("hindsight") && text.contains(HOOK_COMMAND_MARKER));
            if !has_hindsight {
                if let Some(entries) = desired_entries.as_array() {
                    arr.extend(entries.iter().cloned());
                }
            }
        }
    }
}

/// Remove only the Hindsight hook entries from a settings value
fn remove_hooks(existing: &mut serde_json::Value) {
    if let Some(hooks_obj) = existing.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for event_arr in hooks_obj.values_mut() {
            if let Some(arr) = event_arr.as_array_mut() {
                arr.retain(|entry| {
                    let text = serde_json::to_string(entry).unwrap_or_default();
                    !text.contains(HOOK_COMMAND_LEGACY)
                        && !text.contains(HOOK_COMMAND_PREFIX)
                        && !(text.contains("hindsight") && text.contains(HOOK_COMMAND_MARKER))
                });
            }
        }
    }
}

/// Install hooks into a single settings file
fn install_into(settings_path: &PathBuf, force: bool) -> Result<()> {
    let mut value: serde_json::Value = if settings_path.exists() {
        let raw = std::fs::read_to_string(settings_path)?;
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if already_installed(&value) {
        if !force {
            println!("  {} — already installed, skipping.", settings_path.display());
            return Ok(());
        }
        // --force: remove existing entries first, then re-install cleanly
        remove_hooks(&mut value);
    }

    merge_hooks(&mut value);

    // Write back (pretty-printed)
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pretty = serde_json::to_string_pretty(&value)?;
    std::fs::write(settings_path, pretty)?;

    println!("  {} — hooks installed.", settings_path.display());
    Ok(())
}

/// Remove hooks from a single settings file
fn remove_from(settings_path: &PathBuf) -> Result<()> {
    if !settings_path.exists() {
        println!("  {} — not found, skipping.", settings_path.display());
        return Ok(());
    }

    let raw = std::fs::read_to_string(settings_path)?;
    let mut value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));

    if !already_installed(&value) {
        println!("  {} — hooks not present, skipping.", settings_path.display());
        return Ok(());
    }

    remove_hooks(&mut value);
    let pretty = serde_json::to_string_pretty(&value)?;
    std::fs::write(settings_path, pretty)?;

    println!("  {} — hooks removed.", settings_path.display());
    Ok(())
}

const OTEL_ENV_VARS: &[(&str, &str)] = &[
    ("CLAUDE_CODE_ENABLE_TELEMETRY", "1"),
    ("OTEL_METRICS_EXPORTER", "otlp"),
    ("OTEL_LOGS_EXPORTER", "otlp"),
    ("OTEL_EXPORTER_OTLP_PROTOCOL", "http/json"),
    ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:7228"),
];

/// Write OTLP env vars into a settings file.
/// Skips (with a warning) if ANY of the keys are already present.
fn install_otel_env(settings_path: &PathBuf) -> Result<()> {
    let mut value: serde_json::Value = if settings_path.exists() {
        let raw = std::fs::read_to_string(settings_path)?;
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Check for any existing OTLP-related env var.
    let env_obj = value.get("env");
    let already_set: Vec<&str> = OTEL_ENV_VARS
        .iter()
        .filter(|(k, _)| {
            env_obj
                .and_then(|e| e.get(k))
                .is_some()
        })
        .map(|(k, _)| *k)
        .collect();

    if !already_set.is_empty() {
        eprintln!(
            "  Warning: {} already has OTLP env vars set ({}).",
            settings_path.display(),
            already_set.join(", ")
        );
        eprintln!("  Skipping — configure manually to avoid overriding your settings.");
        return Ok(());
    }

    // Merge our vars into the env object.
    let env = value
        .as_object_mut()
        .unwrap()
        .entry("env")
        .or_insert_with(|| serde_json::json!({}));

    for (k, v) in OTEL_ENV_VARS {
        env[k] = serde_json::Value::String(v.to_string());
    }

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pretty = serde_json::to_string_pretty(&value)?;
    std::fs::write(settings_path, pretty)?;

    println!("  {} — OTLP env vars written.", settings_path.display());
    Ok(())
}

pub fn run(remove: bool, status: bool, all_targets: bool, force: bool, otel: bool) -> Result<()> {
    let targets = derive_settings_paths();

    if targets.is_empty() {
        println!("No Claude Code installations found in configured paths.");
        println!("Run 'hindsight paths list' to see configured directories.");
        return Ok(());
    }

    // -- status mode --
    if status {
        println!("Hindsight hook status:\n");
        for (path, label) in &targets {
            let installed = if path.exists() {
                let raw = std::fs::read_to_string(path).unwrap_or_default();
                let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
                already_installed(&v)
            } else {
                false
            };
            let icon = if installed { "" } else { "✗" };
            println!("  {} {} ({})", icon, path.display(), label);
        }
        return Ok(());
    }

    // -- remove mode --
    if remove {
        println!("Removing Hindsight hooks...\n");
        for (path, _) in &targets {
            remove_from(path)?;
        }
        return Ok(());
    }

    // -- install mode --
    let selected: Vec<&(PathBuf, String)> = if all_targets || targets.len() == 1 {
        targets.iter().collect()
    } else {
        // Interactive menu
        println!("Available Claude Code installations:\n");
        for (i, (path, label)) in targets.iter().enumerate() {
            println!("  [{}] {} ({})", i + 1, path.display(), label);
        }
        println!("  [a] All of the above");
        print!("\nSelect: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("a") {
            targets.iter().collect()
        } else {
            match input.parse::<usize>() {
                Ok(n) if n >= 1 && n <= targets.len() => vec![&targets[n - 1]],
                _ => {
                    println!("Invalid selection. Aborted.");
                    return Ok(());
                }
            }
        }
    };

    println!("\nInstalling Hindsight hooks...\n");
    for (path, _) in &selected {
        install_into(path, force)?;
    }

    if otel {
        println!("\nWriting OTLP telemetry env vars...\n");
        for (path, _) in &selected {
            install_otel_env(path)?;
        }
        println!("\nRestart Claude Code for the env vars to take effect.");
    }

    println!("\nDone! Claude Code will now auto-index sessions via hook.");
    println!("Tip: run 'hindsight integrate --status' to verify.");

    Ok(())
}
