<p align="center">
  <img src="site/public/og-image.png" alt="Claude Hindsight" width="600" />
</p>

<h1 align="center">Claude Hindsight</h1>

<p align="center">
  <strong>20/20 hindsight for your Claude Code sessions</strong>
</p>

<p align="center">
  <a href="https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml"><img src="https://github.com/Codestz/claude-hindsight/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://crates.io/crates/claude-hindsight"><img src="https://img.shields.io/crates/v/claude-hindsight.svg" alt="Crates.io" /></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust" /></a>
</p>

<p align="center">
  <a href="https://codestz.github.io/claude-hindsight/">Documentation</a> &middot;
  <a href="https://github.com/Codestz/claude-hindsight/releases">Releases</a> &middot;
  <a href="CHANGELOG.md">Changelog</a>
</p>

---

Claude Hindsight transforms opaque AI coding sessions into crystal-clear insights. It parses the raw JSONL transcripts that [Claude Code](https://claude.com/code) writes to disk and gives you a **full-featured web dashboard**, real-time streaming, and deep analytics — all from a single binary with zero external dependencies.

## Why Hindsight?

- **Understand what happened** — browse every tool call, thinking block, and assistant message in a two-panel inspector
- **Spot problems fast** — errors are highlighted, filterable, and linked to their source nodes
- **Track costs** — per-session and per-model token breakdowns with OpenTelemetry integration
- **Visualize execution** — interactive 3D force-directed graph shows how nodes relate
- **Stay current** — live SSE streaming as Claude works, automatic session discovery
- **Works offline** — everything runs locally, no data leaves your machine

---

## Installation

### Homebrew (recommended)

```bash
brew tap codestz/tap
brew install claude-hindsight
```

### Cargo

```bash
cargo install claude-hindsight
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/Codestz/claude-hindsight/releases) — available for macOS (Apple Silicon & Intel) and Linux (x86_64 & aarch64).

### Build from source

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
make build    # builds frontend + Rust binary → target/release/claude-hindsight
```

Requires Rust 1.75+ and Node.js 20+. Pre-built binaries have **no runtime dependencies** — the web dashboard is embedded in the binary.

---

## Quick Start

```bash
claude-hindsight init                # discover sessions & build index
claude-hindsight integrate           # install Claude Code hooks for auto-indexing
claude-hindsight serve --open        # launch the web dashboard
```

That's it. Open `http://localhost:7227` and explore your sessions.

---

## Web Dashboard

The dashboard is the primary interface. Start it with:

```bash
claude-hindsight serve               # http://localhost:7227
claude-hindsight serve --open        # auto-open browser
claude-hindsight serve --port 8080   # custom port
```

### What you get

**Dashboard** — global overview with session activity sparkline, KPI cards (sessions, projects, cost, errors), top tools, cost by model, most accessed files, MCP server usage, and recent activity feed.

**Session Inspector** — two-panel resizable layout. Left panel shows a flat execution list with color-coded node types, tool names, file chips, and error indicators. Right panel shows full node detail: markdown rendering, syntax-highlighted code (TypeScript, Rust, Python, Go, Bash, HTML, CSS), diff views, image previews, and token usage.

**3D Execution Graph** — interactive force-directed visualization of the session's node hierarchy. Nodes are colored by type, sized by importance. Bloom post-processing, starfield background, click-to-inspect.

**Project Views** — per-project session lists, aggregated analytics, tool usage patterns.

**Activity Timeline** — real-time event feed from Claude Code hooks showing tool calls, errors, and session lifecycle events.

**Live Streaming** — SSE-powered real-time updates. Watch nodes appear as Claude works.

---

## Claude Code Integration

Install lifecycle hooks so Hindsight auto-indexes sessions in real time — no manual `reindex` needed:

```bash
claude-hindsight integrate           # install hooks (interactive)
claude-hindsight integrate --all     # install in all settings files
claude-hindsight integrate --otel    # also enable OpenTelemetry telemetry
claude-hindsight integrate --status  # check what's installed
claude-hindsight integrate --remove  # uninstall hooks
```

Hooks capture: session start/end, tool use, prompt submit, subagent activity, permission requests, task completion, and more. All hooks run async — zero impact on Claude Code performance.

### OpenTelemetry

Enable OTLP telemetry for detailed token-level cost tracking:

```bash
claude-hindsight integrate --otel    # writes env vars to Claude Code settings
claude-hindsight serve               # OTLP receiver runs alongside the dashboard
```

The dashboard then shows per-model cost breakdown, input/output/cache token splits, and cost-per-turn overlays.

---

## Commands

| Command | Description |
|---|---|
| `serve [--open] [--port N]` | Start the web dashboard (recommended) |
| `init` | Discover sessions and build the SQLite index |
| `integrate [--all\|--remove\|--status\|--otel]` | Manage Claude Code lifecycle hooks |
| `reindex [--verbose]` | Sync index with disk, refresh analytics, fix project names |
| `list [--project X] [--errors] [--today] [--last N]` | List sessions with filters |
| `search <query> [--tool X] [--errors]` | Search session content |
| `show <id> [--dashboard]` | Open a session in TUI or web |
| `watch [--dashboard]` | Tail the active session live |
| `export <id> [--output file.html]` | Export session to HTML report |
| `config show\|edit\|reset\|validate` | Manage configuration |
| `paths list\|add\|remove` | Manage scan directories |
| `clean [--db\|--all]` | Reset database or full config |

---

## Terminal UI

For terminal-native workflows, Hindsight also includes a TUI:

```bash
claude-hindsight                     # launch TUI (default when no command given)
claude-hindsight show <session-id>   # open specific session
claude-hindsight watch               # watch active session live
```

Navigate with `j/k`, filter with `/`, switch focus with `Tab`, copy with `y`, quit with `q`.

> **Tip:** For the best experience, use `claude-hindsight serve --open` instead. The web dashboard has richer visualizations, better navigation, and the 3D execution graph.

---

## How It Works

```
Claude Code CLI
     │ writes JSONL transcripts
     v
~/.claude/projects/
  my-app/session-abc123.jsonl
     │
     v
Claude Hindsight
  ├── JSONL Parser (streaming, deduplicating)
  ├── Tree Builder (UUID-based parent/child hierarchy)
  ├── SQLite Indexer (O(1) lookups, aggregated analytics)
  ├── OTLP Receiver (token costs, model breakdown)
  ├── File Watcher (live updates via kqueue/inotify)
  └── SSE Stream (real-time to browser)
     │
     ├── Web Dashboard (axum + React, embedded in binary)
     └── Terminal UI (ratatui)
```

### Key Concepts

- **Session transcripts** — Claude Code writes JSONL to `~/.claude/projects/`. Each line is a node (message, tool call, result, thinking block, etc.)
- **Node hierarchy** — Nodes link via `uuid`/`parent_uuid` to form an execution tree
- **Smart labels** — Nodes are categorized and color-coded: tool calls (amber), thinking (violet), errors (rose), subagents (emerald), tasks (sky)
- **Embedded UI** — The React dashboard is compiled into the binary at build time via `rust-embed`. No Node.js needed at runtime.
- **Auto-update detection** — After upgrading, Hindsight suggests running `reindex` to refresh analytics. Checks GitHub for new versions once per day.

---

## Configuration

```bash
claude-hindsight config show         # print current config
claude-hindsight config edit         # open in $EDITOR
claude-hindsight config validate     # check for errors
claude-hindsight config reset        # restore defaults
```

Config lives at `~/Library/Application Support/claude-hindsight/config.toml` (macOS) or `~/.config/claude-hindsight/config.toml` (Linux).

### Scan directories

```bash
claude-hindsight paths list
claude-hindsight paths add ~/work/.claude/projects --name Work
claude-hindsight paths remove ~/old-projects
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Core & CLI | Rust, clap |
| Web server | axum, tokio |
| Web frontend | React 19, Vite, react-force-graph-3d |
| Terminal UI | ratatui, crossterm |
| Database | SQLite (bundled via rusqlite) |
| Asset embedding | rust-embed |
| File watching | notify (kqueue/inotify) |
| Telemetry | OpenTelemetry (OTLP log receiver) |

---

## Troubleshooting

### After upgrading

Hindsight automatically detects version changes and suggests running `reindex`:

```
  ↑ Updated to v2.3.0 (was v2.2.0). Run hindsight reindex to refresh analytics and fix project names.
```

Just run `claude-hindsight reindex` — it syncs the index with disk, corrects project names, and refreshes all analytics.

### Database errors

```bash
claude-hindsight clean    # delete the database
claude-hindsight init     # rebuild from scratch
```

No data is lost — Hindsight re-parses the original JSONL files.

### Port already in use

```bash
claude-hindsight serve --port 8080       # custom dashboard port
lsof -i :7227                            # find what's using the default port
```

### No sessions found

1. Use [Claude Code](https://claude.com/code) at least once
2. Run `claude-hindsight init`
3. If sessions are in a non-default location: `claude-hindsight paths add /path/to/projects`

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions and coding standards.

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Made with care for the Claude Code community</strong><br>
  <sub>Because every AI decision deserves 20/20 hindsight.</sub>
</p>
