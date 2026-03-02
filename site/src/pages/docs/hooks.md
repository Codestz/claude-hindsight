---
layout: "../../layouts/Docs.astro"
title: "Hooks Setup — Claude Hindsight"
description: "Install Claude Code hooks to automatically index every session without lifting a finger."
---

# Hooks Setup

Set Hindsight up once. It runs automatically forever.

Out of the box, Hindsight is manual: you run `init` or `reindex` to pick up new sessions. With hooks, every Claude Code session is automatically captured, indexed, and streamed into the Activity Timeline the moment it starts — no extra commands needed.

## What hooks do

When you run `claude-hindsight integrate`, it installs two hooks into your Claude Code settings:

1. **`UserPromptSubmit`** — fires when you submit a message to Claude. Hindsight uses this to note that a session has started.
2. **`PreToolUse`** — fires before every tool call. Hindsight uses this to emit real-time OTLP telemetry that populates the Activity Timeline.

It also sets the OTLP environment variables in your Claude Code settings so telemetry flows into the local daemon on port `7228`.

## One-command setup

```bash
claude-hindsight integrate --otel
```

That's it. This command:

- Locates your Claude Code settings file (`~/.claude/settings.json`)
- Injects the two hook entries
- Sets `OTEL_EXPORTER_OTLP_ENDPOINT` and related env vars
- Starts (or registers) the background daemon

> **No Claude Code restart required.** Hooks take effect on the next session you start.

## What gets installed

The integration writes the following into your Claude Code settings:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "claude-hindsight hook session-start"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "claude-hindsight hook tool-use"
          }
        ]
      }
    ]
  }
}
```

And adds OTLP environment variables so every tool call is streamed to the local daemon:

```json
{
  "env": {
    "OTEL_EXPORTER_OTLP_ENDPOINT": "http://localhost:7228",
    "OTEL_SERVICE_NAME": "claude-code"
  }
}
```

## Verifying it works

Check hook status at any time:

```bash
claude-hindsight integrate --status
```

Output example:
```
Hooks:
  ✓ UserPromptSubmit  (claude-hindsight hook session-start)
  ✓ PreToolUse        (claude-hindsight hook tool-use)

OTLP:
  ✓ OTEL_EXPORTER_OTLP_ENDPOINT = http://localhost:7228
  ✓ OTEL_SERVICE_NAME           = claude-code

Daemon:
  ✓ Running on :7228
```

After running a Claude Code session with hooks installed:

1. Open the web dashboard: `claude-hindsight serve --open`
2. Navigate to **Activity** in the top nav
3. You should see a live stream of tool call events from your session

If the Activity page is empty, see [Troubleshooting → Activity Timeline is empty](/docs/troubleshooting#activity-timeline-is-empty).

## Managing hooks

### Remove hooks

```bash
claude-hindsight integrate --remove
```

Removes the Hindsight entries from your Claude Code settings and clears the OTLP environment variables. Does not affect your existing session data or SQLite index.

### Force update

If you upgrade Hindsight and want to refresh the hook configuration:

```bash
claude-hindsight integrate --otel --force
```

`--force` removes any existing Hindsight hook entries before re-installing, ensuring the configuration is current.

### Install without OTLP

If you only want automatic session indexing without the real-time Activity Timeline:

```bash
claude-hindsight integrate
```

This installs only the `UserPromptSubmit` hook. The `PreToolUse` hook and OTLP vars are omitted. The Activity Timeline page will remain empty.

### Install for all users

```bash
claude-hindsight integrate --otel --all
```

Installs into the global Claude Code settings (affects all profiles on the machine) rather than the current user settings.

## The background daemon

The OTLP telemetry is received by the Hindsight daemon, a lightweight background process that listens on port `7228`.

The daemon starts automatically when you run `claude-hindsight serve`. You can also run it standalone:

```bash
claude-hindsight daemon
```

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `7228` | OTLP receiver port |
| `--idle-timeout <SECS>` | `300` | Seconds of inactivity before the daemon exits |

The daemon does not need to be running for session indexing — only for the real-time Activity Timeline. If it's not running when a session starts, the OTLP events are silently dropped (no errors in Claude Code).
