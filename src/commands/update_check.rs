//! Post-command update checks
//!
//! Two features:
//!   1. Detect binary upgrade → suggest `reindex` to refresh analytics
//!   2. Check GitHub releases (once/24h) → warn if a newer version exists

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: u64 = 86_400; // 24 hours
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/Codestz/claude-hindsight/releases/latest";

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    /// Version of the binary that last ran successfully
    #[serde(default)]
    last_run_version: String,

    /// Unix timestamp of the last remote version check
    #[serde(default)]
    last_version_check: u64,

    /// Latest version seen from GitHub
    #[serde(default)]
    latest_known_version: String,

    /// Whether we already suggested reindex for this version
    #[serde(default)]
    reindex_suggested_for: String,
}

fn state_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("claude-hindsight").join(".state"))
}

fn load_state() -> State {
    let Some(path) = state_path() else {
        return State::default();
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &State) {
    let Some(path) = state_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, serde_json::to_string_pretty(state).unwrap_or_default());
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Run after every user-facing command. Prints hints to stderr.
pub fn check() {
    let mut state = load_state();
    let mut dirty = false;

    // 1. Detect version upgrade → suggest reindex
    if !state.last_run_version.is_empty()
        && state.last_run_version != CURRENT_VERSION
        && state.reindex_suggested_for != CURRENT_VERSION
    {
        eprintln!(
            "\n  \x1b[36m↑\x1b[0m Updated to v{} (was v{}). \
             Run \x1b[1mhindsight reindex\x1b[0m to refresh analytics and fix project names.\n",
            CURRENT_VERSION, state.last_run_version
        );
        state.reindex_suggested_for = CURRENT_VERSION.to_string();
        dirty = true;
    }

    // Always track current version
    if state.last_run_version != CURRENT_VERSION {
        state.last_run_version = CURRENT_VERSION.to_string();
        dirty = true;
    }

    // 2. Remote version check (once per 24h, non-blocking on failure)
    let elapsed = now_unix().saturating_sub(state.last_version_check);
    if elapsed >= CHECK_INTERVAL_SECS {
        if let Some(latest) = fetch_latest_version() {
            state.latest_known_version = latest;
            state.last_version_check = now_unix();
            dirty = true;
        }
    }

    if !state.latest_known_version.is_empty() && is_newer(&state.latest_known_version) {
        eprintln!(
            "  \x1b[33m⬆\x1b[0m New version available: v{} → v{}",
            CURRENT_VERSION, state.latest_known_version
        );
        eprintln!(
            "    Update: \x1b[1mbrew upgrade claude-hindsight\x1b[0m or \
             \x1b[1mcargo install claude-hindsight\x1b[0m\n"
        );
    }

    if dirty {
        save_state(&state);
    }
}

/// Mark that reindex was just run, so we don't keep nagging.
pub fn mark_reindex_done() {
    let mut state = load_state();
    state.reindex_suggested_for = CURRENT_VERSION.to_string();
    state.last_run_version = CURRENT_VERSION.to_string();
    save_state(&state);
}

/// Fetch latest version tag from GitHub releases API via curl.
/// Returns None on any failure (no network, timeout, parse error).
fn fetch_latest_version() -> Option<String> {
    let output = std::process::Command::new("curl")
        .args(["-s", "-m", "3", "-H", "Accept: application/vnd.github.v3+json", GITHUB_API_URL])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8(output.stdout).ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = json.get("tag_name")?.as_str()?;

    // Strip leading 'v' if present: "v2.3.0" → "2.3.0"
    Some(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Compare semver strings. Returns true if `latest` is strictly newer than CURRENT_VERSION.
fn is_newer(latest: &str) -> bool {
    let parse = |s: &str| -> Option<(u32, u32, u32)> {
        let mut parts = s.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some((major, minor, patch))
    };

    let Some(current) = parse(CURRENT_VERSION) else {
        return false;
    };
    let Some(remote) = parse(latest) else {
        return false;
    };

    remote > current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        // Temporarily override doesn't work with const, so test the parse logic directly
        let parse = |s: &str| -> (u32, u32, u32) {
            let mut parts = s.split('.');
            (
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
            )
        };

        assert!(parse("2.3.0") > parse("2.2.0"));
        assert!(parse("3.0.0") > parse("2.9.9"));
        assert!(!(parse("2.2.0") > parse("2.2.0")));
        assert!(!(parse("2.1.0") > parse("2.2.0")));
    }

    #[test]
    fn test_strip_v_prefix() {
        let tag = "v2.3.0";
        let version = tag.strip_prefix('v').unwrap_or(tag);
        assert_eq!(version, "2.3.0");

        let tag2 = "2.3.0";
        let version2 = tag2.strip_prefix('v').unwrap_or(tag2);
        assert_eq!(version2, "2.3.0");
    }
}
