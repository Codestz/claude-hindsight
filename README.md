# Claude Hindsight

> **20/20 hindsight for your Claude Code sessions**

[![CI](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml/badge.svg)](https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/claude-hindsight.svg)](https://crates.io/crates/claude-hindsight)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**Claude Hindsight** transforms opaque AI coding sessions into crystal-clear insights. Built for developers who need to **understand**, **debug**, and **analyze** their [Claude Code](https://claude.com/code) workflows.

**[Documentation](https://codestz.github.io/claude-hindsight/)** | **[Changelog](https://github.com/Codestz/claude-hindsight/releases)**

---

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap Codestz/tap
brew install claude-hindsight
```

### Cargo

```bash
cargo install claude-hindsight
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
# binary -> target/release/claude-hindsight
```

**Requirements:** Rust 1.75+, Node.js 20+

Pre-built binaries and Homebrew bottles have **no external dependencies** — the web dashboard is embedded directly in the binary.

---

## Quick Start

```bash
claude-hindsight init          # discover Claude Code sessions & build index
claude-hindsight serve --open  # open web dashboard in browser
claude-hindsight               # launch interactive terminal UI
```

---

## Web Dashboard

```bash
claude-hindsight serve                 # start on http://localhost:7227
claude-hindsight serve --open          # start and open browser
claude-hindsight serve --port 8080     # custom port
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
claude-hindsight                       # launch TUI (default)
claude-hindsight show <session-id>     # open a specific session
claude-hindsight watch                 # watch active session live
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
| `claude-hindsight` | Launch terminal UI (configurable default view) |
| `claude-hindsight init` | Discover sessions and build the SQLite index |
| `claude-hindsight serve` | Start the web dashboard server |
| `claude-hindsight list` | List sessions with filters (`--project`, `--errors`, `--today`, `--last N`, `--with-subagents`) |
| `claude-hindsight show <id>` | Open a session in the TUI or web dashboard (`--dashboard`) |
| `claude-hindsight watch` | Tail the active session in real time |
| `claude-hindsight search <query>` | Search session content (`--tool`, `--errors`) |
| `claude-hindsight export <id>` | Export a session to an HTML report |
| `claude-hindsight reindex` | Sync the SQLite index with disk (prune deleted, discover new) |
| `claude-hindsight config show\|edit\|reset\|validate` | Manage `~/.config/hindsight/config.toml` |
| `claude-hindsight paths list\|add\|remove` | Manage scan directories for Claude session files |

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
claude-hindsight config show       # print current config
claude-hindsight config edit       # open in $EDITOR
claude-hindsight config validate   # check for errors
claude-hindsight config reset      # restore defaults
```

Configurable options include: color themes, key bindings, default startup view, analytics display preferences, cache settings, and scan directories.

### Scan directories

```bash
claude-hindsight paths list                          # show all scan dirs
claude-hindsight paths add ~/work/.claude/projects   # add a new directory
claude-hindsight paths add ~/personal --name Personal # with a display name
claude-hindsight paths remove ~/old-projects         # remove a directory
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

## Troubleshooting

### Database errors after upgrading

Starting with v2.0.1, Hindsight **automatically detects** schema version mismatches and rebuilds the database on the next command. You should see:

```
  Upgrading database schema (v2 -> v3). Rebuilding index...
  Reindexed 142 session(s). Ready!
```

If you still see database errors, manually reset:

```bash
claude-hindsight clean        # delete the database
claude-hindsight init          # rebuild from scratch
```

No session data is lost — Hindsight re-parses the original JSONL files from `~/.claude/projects/`.

### Port already in use

```
Error: Address already in use (os error 48)
```

Another process (or a previous Hindsight instance) is using the port. Either stop it or use a different port:

```bash
claude-hindsight serve --port 8080               # custom web dashboard port
claude-hindsight serve --otel-port 9100           # custom OTLP receiver port
claude-hindsight daemon --port 9100               # custom daemon port
```

To find what's using the default port:

```bash
lsof -i :7227    # web dashboard default
lsof -i :7228    # OTLP receiver / daemon default
```

### No sessions found

```
No sessions found. Run 'hindsight init' after your first Claude Code session.
```

This means Hindsight couldn't find any JSONL session files. Check that:

1. You have used [Claude Code](https://claude.com/code) at least once
2. Session files exist under `~/.claude/projects/`
3. If your sessions are in a non-default location, add the path:
   ```bash
   claude-hindsight paths add /path/to/your/claude/projects
   ```

### Corrupt or locked database

If you see `database is locked` or `database disk image is malformed`:

```bash
claude-hindsight clean        # delete the corrupt database
claude-hindsight init          # rebuild fresh
```

### Web dashboard shows a blank page

The web frontend is embedded in the binary at build time. If you built from source without Node.js, a placeholder page is served instead. To fix:

1. Install Node.js 20+
2. Rebuild: `make build`

Pre-built binaries and Homebrew installs include the full dashboard — no Node.js required.

### OTLP daemon won't start

```
Error: failed to start daemon
```

Check if a daemon is already running:

```bash
ps aux | grep claude-hindsight
```

Kill any stale processes and retry, or use a different port:

```bash
claude-hindsight daemon --port 9100
```

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
