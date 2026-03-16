---
layout: "../../layouts/Docs.astro"
title: "Contributing — Claude Hindsight"
description: "How to set up a dev environment, understand the project structure, and contribute to Hindsight."
---

# Contributing

Contributions are welcome! This guide covers how to set up your development environment, understand the project structure, and submit a pull request.

## Prerequisites

- **Rust** 1.75+ (`rustup` recommended)
- **Node.js** 20+ and **npm** 10+ (for the web dashboard)
- **Git**

## Dev setup

### 1. Clone the repository

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
```

### 2. Install web dependencies

```bash
cd web && npm install && cd ..
```

### 3. Build the web dashboard

The Rust build script (`build.rs`) expects the web assets to be built before compiling. Run:

```bash
cd web && npm run build && cd ..
```

### 4. Build and run Hindsight

```bash
cargo run -- serve
```

For faster iteration during web development, run both servers simultaneously:

```bash
# Terminal 1 — Rust backend
cargo run -- serve

# Terminal 2 — Vite dev server (proxies API calls to :7227)
cd web && npm run dev
```

> **Tip:** The Vite dev server runs on `:5173` and proxies `/api/*` requests to the Rust backend on `:7227`. Hot module replacement works for React components.

### 5. Run tests

```bash
cargo test
```

### 6. Lint

```bash
cargo clippy -- -D warnings
cd web && npm run lint
```

## Project structure

```
claude-hindsight/
  Cargo.toml            Rust workspace manifest
  build.rs              Embeds web/dist/ at compile time
  src/                  Rust source
    main.rs             Entry point, CLI dispatch
    lib.rs              Library exports
    error.rs            Shared error types (thiserror)
    agents/             Agent discovery and parsing
    analyzer/           Session analysis and metrics
    api/                JSON response types
    commands/           CLI subcommand implementations
    config/             Configuration file (TOML) handling
    otel/               OTLP telemetry receiver
    parser/             JSONL session file parser
    search/             Full-text search (SQLite FTS5)
    server/             HTTP server and API route handlers
      routes/           Individual route handlers
    storage/            SQLite index management
    tui/                Terminal UI (ratatui + crossterm)
    watcher/            File watcher for live session tailing
  web/                  React dashboard (Vite + React Router)
    src/
      pages/            React Router page components
      components/       Shared UI components
      lib/              API client, types, utilities
  site/                 Astro docs & landing page
  tests/                Integration tests
  Formula/              Homebrew formula
```

## Code standards

### Rust

- Follow standard Rust idioms; run `cargo clippy` before committing
- Use `thiserror` for error types, `anyhow` for application-level errors
- No `unwrap()` in library code — propagate errors with `?`
- Document public items with `///` doc comments

### TypeScript / React

- Functional components only
- Minimal dependencies — avoid adding new npm packages unless necessary
- No `any` types
- Follow existing patterns for API calls (see `web/src/lib/`)

### Commit messages

Use conventional commits:

```
feat: add session export to CSV
fix: handle empty JSONL files in indexer
docs: update architecture diagram
chore: bump dependency versions
```

## Submitting a pull request

1. Fork the repository and create a branch from `main`
2. Make your changes, following the code standards above
3. Ensure `cargo test` and `cargo clippy -- -D warnings` pass
4. Ensure `cd web && npm run build` succeeds (the rust-embed step requires built assets)
5. Open a pull request with a clear description of the change and why it's needed

## Reporting issues

Use [GitHub Issues](https://github.com/Codestz/claude-hindsight/issues). Please include:

- Claude Hindsight version (`claude-hindsight --version`)
- OS and architecture
- Steps to reproduce
- Expected vs actual behavior

---

## How it works

For those curious about the internals, here's a plain-English overview of how Hindsight works end-to-end.

### Reading session data

Claude Code records every session as a newline-delimited JSON (JSONL) file in `~/.claude/projects/<project-slug>/<session-id>.jsonl`. Each line is a JSON object — a "node" — representing one event: a user message, an assistant response, a tool call, a tool result, a thinking block, or a sub-agent spawn.

Hindsight reads these files using a streaming parser (`src/parser/`) that processes one node at a time, so even sessions with thousands of nodes load quickly.

### The SQLite index

On `claude-hindsight init`, Hindsight scans all configured project directories and builds a local SQLite database at `~/.local/share/claude-hindsight/index.db`.

The schema is intentionally flat to allow fast single-query analytics:

- **`sessions`** — one row per JSONL file: session ID, project, model, token counts, cost, timestamps, error count
- **`tool_uses`** — denormalized tool call records for fast frequency queries
- **`session_fts`** — SQLite FTS5 virtual table for full-text search over message content

`claude-hindsight reindex` uses file modification timestamps to skip sessions that haven't changed, making incremental updates fast.

### Single-binary embed

The web dashboard is a Vite + React Router application built to static files. At Rust compile time, [`rust-embed`](https://github.com/pyros2097/rust-embed) bundles the entire `web/dist/` directory into the binary as read-only byte slices.

At runtime, the embedded HTTP server ([`axum`](https://github.com/tokio-rs/axum)) serves these static assets from memory alongside the JSON API — no Node.js required.

### OTLP telemetry

The `integrate` command installs Claude Code hooks that fire on every tool call. The `PreToolUse` hook calls `claude-hindsight hook tool-use`, which emits an OTLP span to the local daemon on port `7228`.

The daemon (`src/otel/`) is a minimal OTLP receiver that translates incoming spans into activity timeline events and stores them for the dashboard's `/activity` feed.

### Key dependencies

`axum`, `rusqlite`, `rust-embed`, `ratatui`, `tokio`, `serde`, `clap`, `opentelemetry`.
