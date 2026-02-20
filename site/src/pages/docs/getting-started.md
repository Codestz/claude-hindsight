---
layout: "../../layouts/Docs.astro"
title: "Getting Started — Claude Hindsight"
description: "Install and run Claude Hindsight in under a minute."
---

# Getting Started

Claude Hindsight is a single binary that indexes your Claude Code session JSONL files and lets you explore them via a web dashboard, terminal UI, or CLI.

## Prerequisites

- **Claude Code** must be installed and have generated at least one session file (found in `~/.claude/projects/`)
- One of the following install methods:
  - macOS or Linux with [Homebrew](https://brew.sh)
  - Rust toolchain (`cargo`) — version 1.75+
  - A prebuilt binary from the [releases page](https://github.com/Codestz/claude-hindsight/releases)

## Installation

### Homebrew (recommended)

```bash
brew tap Codestz/claude-hindsight
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
# binary is at ./target/release/claude-hindsight
cp target/release/claude-hindsight /usr/local/bin/
```

## First run

### 1. Initialize the index

```bash
claude-hindsight init
```

Claude Hindsight scans `~/.claude/projects/` for JSONL session files and builds a local SQLite index. This is fast even for hundreds of sessions.

### 2. Start the web dashboard

```bash
claude-hindsight serve
```

Opens the dashboard at [http://localhost:7227](http://localhost:7227). You'll see:

- **Dashboard** — global stats, activity chart, error tracking
- **Projects** — per-project session counts and analytics
- **Sessions** — full session list with search and filtering
- **Search** — full-text search across all session content

### 3. Use the terminal UI (optional)

```bash
claude-hindsight list
```

Browse sessions in an interactive TUI inside your terminal.

### 4. Watch a live session

```bash
claude-hindsight watch
```

Tail the most recent active session in real time, showing each tool call and response as it arrives.

## Configuration

By default Claude Hindsight reads from `~/.claude/projects/` and stores its index at `~/.local/share/claude-hindsight/`. You can override the project directory:

```bash
claude-hindsight init --dir /path/to/custom/claude/projects
```

## Keeping the index fresh

Run `claude-hindsight reindex` after long Claude Code sessions to pick up new files:

```bash
claude-hindsight reindex
```

Or add it as a shell alias:

```bash
alias ch-refresh='claude-hindsight reindex && claude-hindsight serve'
```

## Next steps

- [CLI Commands reference](/docs/commands) — all available subcommands and flags
- [Architecture](/docs/architecture) — how Claude Hindsight stores and queries data
