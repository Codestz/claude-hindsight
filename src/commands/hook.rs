//! `claude-hindsight hook <event>` — per-event hook handlers
//!
//! Called automatically by Claude Code via lifecycle hooks.
//! Reads a JSON payload from stdin and stores structured event data.
//! All handlers swallow errors so Claude Code is never blocked.

use crate::error::Result;
use std::io::{self, BufRead};

/// Full hook payload (all fields optional; event-specific fields may be absent)
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct FullHookPayload {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub hook_event_name: Option<String>,
    // PreToolUse / PostToolUse / PostToolUseFailure
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,
    pub error: Option<String>,
    pub is_interrupt: Option<bool>,
    pub tool_use_id: Option<String>,
    // SubagentStart / SubagentStop
    pub agent_type: Option<String>,
    pub agent_name: Option<String>,
    // PermissionRequest
    pub action: Option<String>,
    // TaskCompleted
    pub result: Option<String>,
    // WorktreeCreate / WorktreeRemove
    pub worktree_path: Option<String>,
    pub branch: Option<String>,
    // ConfigChange
    pub key: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

/// Read and deserialize stdin payload, returning a default on any error.
pub fn read_payload() -> FullHookPayload {
    let stdin = io::stdin();
    let mut raw = String::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                raw.push_str(&l);
                raw.push('\n');
            }
            Err(_) => break,
        }
    }
    serde_json::from_str(raw.trim()).unwrap_or_default()
}

// ── OTLP daemon auto-spawn ─────────────────────────────────────────────────────

/// Ensure the OTLP receiver is listening on port 7228.
/// If nothing is bound, spawn a detached `<binary> daemon` process.
fn ensure_otlp_daemon() {
    use std::net::TcpStream;
    use std::time::Duration;

    let port: u16 = crate::config::Config::load()
        .map(|c| c.telemetry.otel_port)
        .unwrap_or(7228);

    // Quick TCP connect check — if something is already listening, do nothing
    if TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(200),
    )
    .is_ok()
    {
        return;
    }

    // Resolve our own binary path
    let bin = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };

    // Spawn detached daemon with idle timeout — auto-exits after 10 min of inactivity
    let _ = std::process::Command::new(bin)
        .arg("daemon")
        .arg("--port")
        .arg(port.to_string())
        .arg("--idle-timeout")
        .arg("600")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn(); // fire-and-forget
}

// ── Delegating events (session lifecycle already handled by hook_index) ────────

pub fn run_session_start() -> Result<()> {
    ensure_otlp_daemon();
    crate::commands::hook_index::run()
}

pub fn run_stop() -> Result<()> {
    crate::commands::hook_index::run()
}

pub fn run_session_end() -> Result<()> {
    crate::commands::hook_index::run()
}

// ── Tool-level events ─────────────────────────────────────────────────────────

pub fn run_pre_tool_use() -> Result<()> {
    ensure_otlp_daemon();
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let input_str = p.tool_input.as_ref().map(|v| v.to_string());
        let _ = idx.insert_hook_tool_event(
            sid,
            "PreToolUse",
            p.tool_name.as_deref(),
            input_str.as_deref(),
            None,
            None,
            None,
            p.tool_use_id.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

pub fn run_post_tool_use() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let input_str = p.tool_input.as_ref().map(|v| v.to_string());
        let result_str = p.tool_response.as_ref().map(|v| v.to_string());
        let _ = idx.insert_hook_tool_event(
            sid,
            "PostToolUse",
            p.tool_name.as_deref(),
            input_str.as_deref(),
            result_str.as_deref(),
            None,
            p.is_interrupt,
            p.tool_use_id.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

pub fn run_post_tool_use_failure() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let input_str = p.tool_input.as_ref().map(|v| v.to_string());
        let _ = idx.insert_hook_tool_event(
            sid,
            "PostToolUseFailure",
            p.tool_name.as_deref(),
            input_str.as_deref(),
            None,
            p.error.as_deref(),
            p.is_interrupt,
            p.tool_use_id.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

// ── Subagent events ───────────────────────────────────────────────────────────

pub fn run_subagent_start() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let _ = idx.insert_hook_subagent_event(
            sid,
            "SubagentStart",
            p.agent_type.as_deref(),
            p.agent_name.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

pub fn run_subagent_stop() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let _ = idx.insert_hook_subagent_event(
            sid,
            "SubagentStop",
            p.agent_type.as_deref(),
            p.agent_name.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

// ── Compaction ────────────────────────────────────────────────────────────────

pub fn run_pre_compact() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let _ = idx.insert_hook_compaction_event(sid, p.hook_event_name.as_deref());
    }
    Ok(())
}

// ── Permission request ────────────────────────────────────────────────────────

pub fn run_permission_request() -> Result<()> {
    let p = read_payload();
    let sid = match p.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        let input_str = p.tool_input.as_ref().map(|v| v.to_string());
        let _ = idx.insert_hook_permission_event(
            sid,
            p.tool_name.as_deref(),
            input_str.as_deref(),
            p.cwd.as_deref(),
        );
    }
    Ok(())
}

// ── Lifecycle catch-all events ────────────────────────────────────────────────

fn run_lifecycle(event_name: &str, payload: FullHookPayload) -> Result<()> {
    let sid = match payload.session_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };
    if let Ok(idx) = crate::storage::SessionIndex::new() {
        // Encode event-specific fields as a JSON attributes blob
        let attrs = serde_json::json!({
            "result": payload.result,
            "worktree_path": payload.worktree_path,
            "branch": payload.branch,
            "key": payload.key,
            "old_value": payload.old_value,
            "new_value": payload.new_value,
            "cwd": payload.cwd,
        });
        let attrs_str = attrs.to_string();
        let _ = idx.insert_hook_lifecycle_event(sid, event_name, Some(&attrs_str));
    }
    Ok(())
}

pub fn run_task_completed() -> Result<()> {
    run_lifecycle("TaskCompleted", read_payload())
}

pub fn run_worktree_create() -> Result<()> {
    run_lifecycle("WorktreeCreate", read_payload())
}

pub fn run_worktree_remove() -> Result<()> {
    run_lifecycle("WorktreeRemove", read_payload())
}

pub fn run_config_change() -> Result<()> {
    run_lifecycle("ConfigChange", read_payload())
}
