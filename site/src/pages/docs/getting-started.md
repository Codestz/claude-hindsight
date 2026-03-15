---
layout: "../../layouts/Docs.astro"
title: "Introduction — Claude Hindsight"
description: "What Claude Hindsight gives you, how to install it, and how to get the most out of it in five minutes."
---

# Introduction

Claude Hindsight is the complete monitoring system for Claude Code. Every time Claude runs a task — reading files, editing code, running tests, spawning sub-agents — Hindsight captures everything and makes it explorable through a rich web dashboard.

> **We recommend using the web dashboard** (`claude-hindsight serve --open`) for the best experience. The TUI is useful for quick terminal browsing, but the dashboard provides 3D graph visualization, image previews, syntax-highlighted code, resizable panels, and the full suite of v2.2 features.

## What you get

- **Two-panel session inspector** — compact execution list on the left, full node detail on the right, with resizable split layout
- **3D force-directed graph** — visualize entire sessions as interactive WebGL graphs with bloom lighting
- **Smart tool rendering** — Read, Edit, Bash, Write each get specialized displays with syntax highlighting for 8+ languages
- **Image previews** — screenshots embedded in tool results render inline with expand-to-lightbox
- **Cost & error tracking** — track token costs per turn, per model, per project across all sessions
- **Real-time streaming** — watch sessions live as Claude works, powered by SSE
- **Set up once with hooks** — one command installs Claude Code hooks so every future session is captured automatically

---

## Install

### Homebrew (recommended — macOS & Linux)

```bash
brew tap codestz/tap
brew install claude-hindsight
```

### cargo install

```bash
cargo install claude-hindsight
```

### Prebuilt binary

Download the latest binary for your platform from the [GitHub releases page](https://github.com/Codestz/claude-hindsight/releases) and place it somewhere on your `$PATH`.

### Build from source

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
cargo build --release
cp target/release/claude-hindsight /usr/local/bin/
```

---

## First run

### 1. Build the index

```bash
claude-hindsight init
```

Scans `~/.claude/projects/` and builds a local SQLite index of your sessions. Fast even for hundreds of sessions.

### 2. Open the dashboard

```bash
claude-hindsight serve --open
```

Opens the web portal at [http://localhost:7227](http://localhost:7227). You'll see a global stats dashboard, your full session list, and per-session execution trees.

---

## Recommended: set up hooks

With the default setup, you need to run `reindex` manually after each Claude Code session to pick up new data.

The better way:

```bash
claude-hindsight integrate --otel
```

This one command installs two Claude Code hooks and starts the OTLP daemon. From that point on:

- Every new session is indexed automatically the moment it starts
- Tool calls stream into the **Activity Timeline** in real time

See the [Hooks Setup](/docs/hooks) page for details, including how to verify the installation and manage hooks.

---

## What's inside the dashboard

| Page | What it shows |
|------|---------------|
| **Dashboard** | Global stats, cost by model, conversations/day chart, top tools, recent sessions |
| **Sessions** | Full list with search, project filter, error filter |
| **Session Detail** | Two-panel inspector with LIST/GRAPH views, image previews, timeline scrubber |
| **Projects** | Per-project session count, size, activity |
| **Activity** | Real-time OTLP event feed with error tracking (requires hooks) |
| **Prompts** | Scored user prompts across all sessions |
| **Skills** | Skill files discovered across all scopes |
| **Agents** | Agent definitions discovered across all scopes |

Full tour: [Web Dashboard](/docs/dashboard)

---

## A note on how it works

Hindsight reads the JSONL files that Claude Code writes to `~/.claude/projects/`. It builds a local SQLite index from those files and serves a web portal from that index. Everything stays on your machine — no network requests, no accounts, no cloud.

For technical details (crate structure, OTLP internals, build system), see [Contributing](/docs/contributing#how-it-works).

---

## Next steps

- [Hooks Setup](/docs/hooks) — set up automatic indexing in one command
- [Web Dashboard](/docs/dashboard) — full tour of every portal page
- [CLI Commands](/docs/commands) — complete CLI reference
- [Configuration](/docs/configuration) — config file, scan paths, ports
