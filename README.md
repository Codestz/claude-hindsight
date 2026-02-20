# Claude Hindsight

> **20/20 hindsight for your Claude Code sessions**

[![CI](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml/badge.svg)](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/hindsight.svg)](https://crates.io/crates/hindsight)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**Claude Hindsight** transforms opaque AI coding sessions into crystal-clear insights. Built for developers who need to **understand**, **debug**, and **analyze** their [Claude Code](https://claude.com/code) workflows.

**[Documentation](https://codestz.github.io/claude-hindsight/)** | **[Changelog](https://github.com/Codestz/claude-hindsight/releases)**

---

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap Codestz/hindsight
brew install hindsight
```

### Cargo

```bash
cargo install hindsight
```

### Pre-built binaries

Download from the [GitHub Releases](https://github.com/Codestz/claude-hindsight/releases) page. Available for:
- macOS (Apple Silicon & Intel)
- Linux (x86_64 & aarch64)

### Build from source

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
make build            # builds frontend + Rust binary
# binary -> target/release/hindsight
```

**Requirements:** Rust 1.75+, Node.js 20+

Pre-built binaries and Homebrew bottles have **no external dependencies** — the web dashboard is embedded directly in the binary.

---

## Quick Start

```bash
hindsight init          # discover Claude Code sessions & build index
hindsight serve --open  # open web dashboard in browser
hindsight               # launch interactive terminal UI
```

---

## Web Dashboard

```bash
hindsight serve                 # start on http://localhost:7227
hindsight serve --open          # start and open browser
hindsight serve --port 8080     # custom port
```

A self-contained web dashboard with:
- **Global analytics** — sessions over time, tool usage, error tracking
- **Project views** — per-project session lists, top tools, most accessed files, MCP server usage
- **Session detail** — full execution tree with smart node labels, diff viewer, raw JSON
- **Live feed** — real-time updates via SSE as Claude works
- **Error navigation** — jump directly to failures
- **Subagent tracking** — see which models subagents use and their source directories

---

## Terminal UI

```bash
hindsight                       # launch TUI (default)
hindsight show <session-id>     # open a specific session
hindsight watch                 # watch active session live
```

**Keyboard shortcuts:**
- `j/k` or `Up/Down` — navigate tree
- `/` — filter by node type (`user`, `tool_use`, `thinking`, ...)
- `n/N` — jump between filtered nodes
- `Tab` — switch focus (tree / details)
- `Ctrl+d/u` — half-page scroll
- `g/G` — top / bottom
- `y` — copy selected node content to clipboard
- `q` — quit

---

## Commands

| Command | Description |
|---|---|
| `hindsight` | Launch terminal UI (configurable default view) |
| `hindsight init` | Discover sessions and build the SQLite index |
| `hindsight serve` | Start the web dashboard server |
| `hindsight list` | List sessions with filters (`--project`, `--errors`, `--today`, `--last N`, `--with-subagents`) |
| `hindsight show <id>` | Open a session in the TUI or web dashboard (`--dashboard`) |
| `hindsight watch` | Tail the active session in real time |
| `hindsight search <query>` | Search session content (`--tool`, `--errors`) |
| `hindsight export <id>` | Export a session to an HTML report |
| `hindsight reindex` | Sync the SQLite index with disk (prune deleted, discover new) |
| `hindsight config show\|edit\|reset\|validate` | Manage `~/.config/hindsight/config.toml` |
| `hindsight paths list\|add\|remove` | Manage scan directories for Claude session files |

---

## How It Works

```
┌─────────────────┐
│ Claude Code CLI  │
└────────┬────────┘
         │ writes JSONL
         v
┌────────────────────────────────┐
│ ~/.claude/projects/            │
│   my-app/                      │
│     session-abc123.jsonl       │
└────────┬───────────────────────┘
         │ parsed by
         v
┌────────────────────────────────┐
│ Claude Hindsight               │
│  • JSONL Parser                │
│  • Tree Builder (UUID-based)   │
│  • SSE Stream Deduplication    │
│  • SQLite Indexer              │
│  • Real-time File Watcher      │
└────────┬───────────────────────┘
         │ exposed via
         v
┌──────────────────┐   ┌──────────────────┐
│ Web Dashboard    │   │ Terminal UI       │
│ (axum + Next.js) │   │ (ratatui)         │
│ embedded in bin  │   │                   │
└──────────────────┘   └──────────────────┘
```

### Key Concepts

- **JSONL Format** — Claude Code writes newline-delimited JSON to `~/.claude/projects/`
- **UUID Hierarchy** — nodes linked by `uuid` / `parent_uuid` relationships form the execution tree
- **Smart Labels** — nodes are categorized and color-coded by type (tool calls, thinking, errors, subagents, compact boundaries)
- **SSE Deduplication** — streaming events are deduplicated so each message appears once in the tree
- **File Watching** — live updates via filesystem notifications (no polling)
- **SQLite Index** — O(1) session lookups, aggregated analytics, no repeated file parsing
- **Embedded UI** — the entire Next.js dashboard is compiled into the binary at release time via `rust-embed`
- **Multiple Scan Dirs** — configure additional Claude session directories beyond the default `~/.claude/projects/`

---

## Configuration

Configuration lives at `~/.config/hindsight/config.toml`. Manage it with:

```bash
hindsight config show       # print current config
hindsight config edit       # open in $EDITOR
hindsight config validate   # check for errors
hindsight config reset      # restore defaults
```

Configurable options include: color themes, key bindings, default startup view, analytics display preferences, cache settings, and scan directories.

### Scan directories

```bash
hindsight paths list                          # show all scan dirs
hindsight paths add ~/work/.claude/projects   # add a new directory
hindsight paths add ~/personal --name Personal # with a display name
hindsight paths remove ~/old-projects         # remove a directory
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| CLI & core | Rust 1.75+, clap |
| Terminal UI | ratatui, crossterm |
| Web server | axum, tokio |
| Web frontend | Next.js 15, React 19 |
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
  <strong>Made with care for the Claude Code community</strong><br>
  <sub>Because every AI decision deserves 20/20 hindsight.</sub>
</p>
