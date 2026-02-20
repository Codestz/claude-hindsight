---
layout: "../../layouts/Docs.astro"
title: "Getting Started — Hindsight"
description: "Install and run Hindsight in under a minute."
---

# Getting Started

Hindsight is a single binary that indexes your Claude Code session JSONL files and lets you explore them via a web dashboard, terminal UI, or CLI.

## Prerequisites

- **Claude Code** must be installed and have generated at least one session file (found in `~/.claude/projects/`)
- One of the following install methods:
  - macOS or Linux with [Homebrew](https://brew.sh)
  - Rust toolchain (`cargo`) — version 1.75+
  - A prebuilt binary from the [releases page](https://github.com/Codestz/claude-hindsight/releases)

## Installation

### Homebrew (recommended)

```bash
brew tap Codestz/hindsight
brew install hindsight
```

### cargo install

```bash
cargo install hindsight
```

### Prebuilt binary

Download the latest binary for your platform from the [GitHub releases page](https://github.com/Codestz/claude-hindsight/releases) and place it somewhere on your `$PATH`.

### Build from source

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
cargo build --release
# binary is at ./target/release/hindsight
cp target/release/hindsight /usr/local/bin/
```

## First run

### 1. Initialize the index

```bash
hindsight init
```

Hindsight scans `~/.claude/projects/` for JSONL session files and builds a local SQLite index. This is fast even for hundreds of sessions.

### 2. Start the web dashboard

```bash
hindsight serve
```

Opens the dashboard at [http://localhost:7227](http://localhost:7227). You'll see:

- **Dashboard** — global stats, token usage, cost totals, activity chart
- **Projects** — per-project session counts and analytics
- **Sessions** — full session list with search and filtering
- **Search** — full-text search across all session content

### 3. Use the terminal UI (optional)

```bash
hindsight list
```

Browse sessions in an interactive TUI inside your terminal.

### 4. Watch a live session

```bash
hindsight watch
```

Tail the most recent active session in real time, showing each tool call and response as it arrives.

## Configuration

By default Hindsight reads from `~/.claude/projects/` and stores its index at `~/.local/share/hindsight/`. You can override the project directory:

```bash
hindsight init --dir /path/to/custom/claude/projects
```

## Keeping the index fresh

Run `hindsight reindex` after long Claude Code sessions to pick up new files:

```bash
hindsight reindex
```

Or add it as a shell alias:

```bash
alias hs-refresh='hindsight reindex && hindsight serve'
```

## Next steps

- [CLI Commands reference](/docs/commands) — all available subcommands and flags
- [Architecture](/docs/architecture) — how Hindsight stores and queries data
