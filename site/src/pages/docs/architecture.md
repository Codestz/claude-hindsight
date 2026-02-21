---
layout: "../../layouts/Docs.astro"
title: "Architecture — Claude Hindsight"
description: "How Claude Hindsight stores and queries Claude Code session data."
---

# Architecture

Hindsight is designed around a single key constraint: **zero runtime dependencies**. Everything — the web dashboard, the static assets, and the data layer — ships in one binary.

## Session files (JSONL)

Claude Code records every session as a newline-delimited JSON file (JSONL) in:

```
~/.claude/projects/<project-slug>/<session-id>.jsonl
```

Each line is a JSON object representing one "node" — a message, tool call, tool result, or system event. Nodes form a tree structure: assistant messages contain tool use, tool results reference the call, sub-agents nest inside the parent session.

**Example node types:**

| Type | Description |
|------|-------------|
| `human` | User message |
| `assistant` | Claude response |
| `tool_use` | Claude invoking a tool |
| `tool_result` | Tool output |
| `summary` | Auto-generated session summary |

Hindsight parses these files using a streaming JSONL reader so even very large sessions (tens of thousands of nodes) load quickly.

## SQLite index

On `claude-hindsight init`, Claude Hindsight scans the project directories and builds a local SQLite database (default: `~/.local/share/claude-hindsight/index.db`).

The schema includes:

- **`sessions`** — one row per JSONL file: session ID, project name, file path, token counts, cost estimate, first message, model, timestamps, error count
- **`tool_uses`** — denormalized tool call records for fast frequency queries
- **`session_fts`** — SQLite FTS5 virtual table for full-text search over message content

The index is kept deliberately flat (not deeply normalized) to allow fast single-query analytics aggregations across hundreds of sessions.

### Incremental updates

`claude-hindsight reindex` uses file modification timestamps to skip sessions that haven't changed since the last index pass. Only new or modified JSONL files are re-parsed.

## Single-binary embed

The web dashboard is a Next.js application built to static files (`next build`). At Rust compile time, [`rust-embed`](https://github.com/pyros2097/rust-embed) bundles the entire `web/out/` directory into the binary as read-only byte slices.

At runtime, the embedded HTTP server (built with [`axum`](https://github.com/tokio-rs/axum)) serves these static assets from memory alongside the JSON API.

```
┌─────────────────────────────────┐
│     claude-hindsight binary     │
│                                 │
│  ┌───────────┐ ┌─────────────┐  │
│  │ axum HTTP │ │ rust-embed  │  │
│  │  server   │ │ web/out/*   │  │
│  └─────┬─────┘ └─────────────┘  │
│        │                        │
│  ┌─────▼─────────────────────┐  │
│  │   JSON API (/api/*)       │  │
│  │   SQLite via rusqlite     │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
```

## API surface

The server exposes a minimal JSON API consumed by the dashboard:

| Endpoint | Description |
|----------|-------------|
| `GET /api/analytics` | Global session aggregates |
| `GET /api/sparkline?days=N` | Daily session counts for the last N days |
| `GET /api/sessions` | Paginated session list |
| `GET /api/sessions/:id` | Single session metadata |
| `GET /api/sessions/:id/tree` | Full node tree for a session |
| `GET /api/projects` | Per-project stats |
| `GET /api/search?q=<query>` | Full-text search results |

## Rust crate structure

```
src/
  main.rs          Entry point, CLI dispatch
  lib.rs           Library exports
  error.rs         Shared error types (thiserror)
  server/          HTTP server and API handlers
    mod.rs
    routes.rs
    analytics.rs
  storage/         JSONL parsing and SQLite index
    mod.rs
    index.rs
    parser.rs
  search/          Full-text search (FTS5)
    mod.rs
  api/             JSON response types
    responses.rs
  commands/        CLI subcommands
    mod.rs
    export.rs
    reindex.rs
    watch.rs
  tui/             Terminal UI (ratatui)
    mod.rs
    app.rs
```

Key dependencies: `axum`, `rusqlite`, `rust-embed`, `ratatui`, `tokio`, `serde`, `clap`.
