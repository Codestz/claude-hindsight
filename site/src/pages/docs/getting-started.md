---
layout: "../../layouts/Docs.astro"
title: "Introduction — Claude Hindsight"
description: "What Claude Hindsight gives you, how to install it, and how to get the most out of it in five minutes."
---

# Introduction

Claude Hindsight is the observability layer for Claude Code. Every time Claude runs a task for you — reading files, editing code, running tests — Hindsight captures the complete picture and makes it explorable.

## What you get

- **See every tool call** — the full execution tree: thinking blocks, every Glob, Read, Edit, and Bash call, with inputs and outputs
- **Catch errors instantly** — failed tool calls are flagged so you can jump straight to what went wrong
- **Track token usage** — input, output, and cached tokens per session, cost estimates across all projects
- **Set up once with hooks** — one command installs Claude Code hooks so every future session is captured automatically, no manual steps needed

---

## Install

### Homebrew (recommended — macOS & Linux)

```bash
brew tap Codestz/homebrew-tap
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
| **Dashboard** | Global stats: session count, total cost, error count, recent sessions, top tools |
| **Sessions** | Full list with search, project filter, and error filter |
| **Session Detail** | Complete execution tree for one session — every node, expandable |
| **Projects** | Per-project analytics |
| **Activity** | Real-time OTLP event feed (requires hooks) |
| **Skills** | Skill files discovered in your project directories |
| **Agents** | Agent definitions discovered in your project directories |

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
