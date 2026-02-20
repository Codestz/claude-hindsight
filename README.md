# Claude Hindsight

> **20/20 hindsight for your Claude Code sessions**

[![CI](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml/badge.svg)](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Claude Hindsight** transforms opaque AI coding sessions into crystal-clear insights. Built for developers who need to **understand**, **debug**, and **optimize** their [Claude Code](https://claude.com/code) workflows.

---

## Installation

### Homebrew (macOS & Linux — recommended)

```bash
brew tap Codestz/hindsight
brew install hindsight
```

### Cargo

```bash
cargo install hindsight
```

### Build from source

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
make build            # builds frontend + Rust binary
# binary → target/release/hindsight
```

**Requirements for building from source:** Rust 1.75+, Node.js 20+

Pre-built binaries and Homebrew bottles have **no external dependencies** — the web dashboard is embedded directly in the binary.

---

## Quick Start

```bash
hindsight init          # discover Claude Code sessions
hindsight serve --open  # open web dashboard in browser
hindsight              # launch interactive terminal UI
```

---

## Commands

### `hindsight serve` — Web Dashboard

```bash
hindsight serve                 # start on http://localhost:7227
hindsight serve --open          # start and open browser
hindsight serve --port 8080     # custom port
```

A self-contained web dashboard with:
- **Global analytics** — sessions over time, tool usage, token spend
- **Session explorer** — browse and search all sessions
- **Session detail** — full execution tree, diff view, raw JSON, replay
- **Live feed** — real-time updates as Claude works
- **Error navigation** — jump directly to failures

### `hindsight` — Terminal UI

```bash
hindsight show <session-id>     # open a specific session
hindsight watch                 # watch active session live
```

**Keyboard shortcuts:**
- `j/k` or `↑↓` — navigate tree
- `/` — filter by node type (`user`, `tool_use`, `thinking`, …)
- `n/N` — jump between filtered nodes
- `Tab` — switch focus (tree ↔ details)
- `Ctrl+d/u` — half-page scroll
- `g/G` — top / bottom
- `q` — quit

### `hindsight list` — Browse Sessions

```bash
hindsight list                  # all sessions
hindsight list --project my-app # filter by project
hindsight list --errors         # sessions with failures
hindsight list --last 10        # most recent 10
hindsight list --today          # today only
```

### `hindsight reindex` — Rebuild Index

```bash
hindsight reindex               # sync SQLite index with disk
hindsight reindex --verbose     # verbose output
```

Run after upgrading or if analytics feel stale. Prunes deleted sessions, discovers new ones, re-parses changed files.

---

## How It Works

```
┌─────────────────┐
│ Claude Code CLI │
└────────┬────────┘
         │ writes JSONL
         ↓
┌────────────────────────────────┐
│ ~/.claude/projects/            │
│   my-app/                      │
│     session-abc123.jsonl       │
└────────┬───────────────────────┘
         │ parsed by
         ↓
┌────────────────────────────────┐
│ Claude Hindsight               │
│  • JSONL Parser                │
│  • Tree Builder (UUID-based)   │
│  • SQLite Indexer              │
│  • Real-time File Watcher      │
└────────┬───────────────────────┘
         │ exposed via
         ↓
┌──────────────────┐   ┌──────────────────┐
│ Web Dashboard    │   │ Terminal UI      │
│ (axum + Next.js) │   │ (ratatui)        │
│ embedded in bin  │   │                  │
└──────────────────┘   └──────────────────┘
```

### Key Concepts

- **JSONL Format** — Claude Code writes newline-delimited JSON to `~/.claude/projects/`
- **UUID Hierarchy** — nodes linked by `uuid` / `parent_uuid` relationships form the execution tree
- **File Watching** — live updates via filesystem notifications (no polling)
- **SQLite Index** — O(1) session lookups, aggregated analytics, no repeated file parsing
- **Embedded UI** — the entire Next.js dashboard is compiled into the binary at release time via `rust-embed`

---

## Tech Stack

| Layer | Technology |
|---|---|
| CLI & core | Rust 1.75+, clap |
| Terminal UI | ratatui, crossterm |
| Web server | axum, tokio |
| Web frontend | Next.js 15, React 19, Tailwind CSS |
| Asset embedding | rust-embed (frontend baked into binary) |
| Database | SQLite (bundled via rusqlite) |
| File watching | notify |

---

## Architecture — Single Binary

Release builds embed the full Next.js static bundle using `rust-embed`. There is no separate installation step, no Node.js on the target machine, and no `web/out/` directory to manage. The binary is self-contained.

`build.rs` handles the frontend build automatically:
- If `web/out/` exists → embeds it as-is (fast incremental builds)
- If missing and Node.js is available → runs `npm install && npm run build`
- If Node.js is absent → embeds a placeholder page; API still works fully

---

## Security

- **Local-only** — no network requests, no telemetry, no data leaves your machine
- **Read-only** — Hindsight never modifies sessions or code
- **No credentials** — does not read `.env` files or access API keys
- **SQLite** — ACID-compliant storage with data integrity guarantees
- **Rust** — memory-safe by design

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding standards, and how to open a pull request.

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Made with ❤️ for the Claude Code community</strong><br>
  <sub>Because every AI decision deserves 20/20 hindsight.</sub>
</p>
