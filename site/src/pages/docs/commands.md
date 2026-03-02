---
layout: "../../layouts/Docs.astro"
title: "CLI Commands — Claude Hindsight"
description: "Full CLI reference for all Claude Hindsight v2 subcommands and flags."
---

# CLI Commands

Complete reference for the `claude-hindsight` command-line interface.

Use `claude-hindsight --help` or `claude-hindsight <command> --help` at any time for inline help. `--version` / `-V` prints the version string.

---

## `init`

Scan Claude session directories and build the SQLite index.

```bash
claude-hindsight init [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--dir <PATH>` | `~/.claude/projects` | Root directory containing project subdirectories |
| `--db <PATH>` | `~/.local/share/claude-hindsight/index.db` | Path to the SQLite database |

```bash
claude-hindsight init --dir ~/work/claude-projects
```

---

## `reindex`

Incrementally update the index with new or changed session files.

```bash
claude-hindsight reindex [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--dir <PATH>` | (from config) | Override scan directory |
| `--full` | false | Drop and rebuild the entire index |

Uses file modification timestamps to skip unchanged sessions. Run this after sessions if hooks are not installed.

---

## `serve`

Start the web dashboard server.

```bash
claude-hindsight serve [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `7227` | TCP port for the web dashboard |
| `--host <HOST>` | `127.0.0.1` | Bind address |
| `--open` | false | Open browser automatically on start |
| `--otel-port <PORT>` | `7228` | Port for the OTLP receiver (also starts the daemon) |

```bash
claude-hindsight serve --open
claude-hindsight serve --port 8080 --otel-port 8081
```

---

## `daemon`

Run the OTLP telemetry receiver as a standalone background process.

```bash
claude-hindsight daemon [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `7228` | Port to listen for OTLP spans |
| `--idle-timeout <SECS>` | `300` | Exit after N seconds with no incoming spans |

The daemon receives telemetry from Claude Code hooks and feeds the Activity Timeline. It starts automatically when you run `serve`, or you can run it standalone for always-on monitoring.

```bash
claude-hindsight daemon --port 7228
```

---

## `integrate`

Install or manage Claude Code hooks and OTLP configuration.

```bash
claude-hindsight integrate [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--otel` | Also install OTLP env vars and `PreToolUse` hook |
| `--force` | Remove existing Hindsight hooks before reinstalling |
| `--remove` | Remove all Hindsight hooks and OTLP env vars |
| `--status` | Print current hook and daemon status |
| `--all` | Write to global Claude Code settings (all users) |

```bash
# Full setup (recommended)
claude-hindsight integrate --otel

# Check what's installed
claude-hindsight integrate --status

# Upgrade hooks after a Hindsight update
claude-hindsight integrate --otel --force

# Remove everything
claude-hindsight integrate --remove
```

---

## `list`

Launch the interactive terminal UI for browsing sessions.

```bash
claude-hindsight list [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--project <NAME>` | (all) | Filter to a specific project |
| `--limit <N>` | 50 | Maximum sessions to load |
| `--errors` | false | Show only sessions with errors |
| `--today` | false | Show only sessions from today |
| `--with-subagents` | false | Show only sessions that spawned subagents |
| `--last <N>` | — | Show the N most recent sessions |

**TUI keybindings:**

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Open session detail |
| `/` | Filter sessions |
| `q` | Quit |

```bash
claude-hindsight list --errors --today
claude-hindsight list --last 10
```

---

## `show`

Print details of a single session to stdout.

```bash
claude-hindsight show <SESSION_ID>
```

Prints session metadata, message count, and a text tree of tool calls.

```bash
claude-hindsight show abc12345
```

---

## `watch`

Tail the most recently modified session file in real time.

```bash
claude-hindsight watch [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--session <ID>` | (latest) | Watch a specific session by ID |
| `--project <NAME>` | (all) | Restrict to a project |

Follows the JSONL file from the end, printing each new node as it arrives via SSE.

```bash
claude-hindsight watch
claude-hindsight watch --project my-app
```

---

## `search`

Search session content from the command line.

```bash
claude-hindsight search <QUERY> [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--project <NAME>` | (all) | Restrict search to a project |
| `--limit <N>` | 20 | Max results |
| `--json` | false | Output as JSON |

```bash
claude-hindsight search "authentication middleware" --project api-server
```

---

## `export`

Export session data to a file.

```bash
claude-hindsight export [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--output <PATH>` | `./hindsight-export.json` | Output file path |
| `--project <NAME>` | (all) | Export only a specific project |
| `--format <FMT>` | `json` | Output format: `json` or `csv` |

```bash
claude-hindsight export --output my-sessions.json
claude-hindsight export --format csv --project my-app --output sessions.csv
```

---

## `clean`

Remove Hindsight data files.

```bash
claude-hindsight clean [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--db` | Remove the SQLite index only (preserves original JSONL files) |
| `--all` | Remove all Hindsight data (index + config) |
| `--yes` | Skip confirmation prompt |

```bash
# Reset the index (e.g. after a schema migration)
claude-hindsight clean --db

# Nuclear option — remove everything
claude-hindsight clean --all --yes
```

> **Note:** `clean` never touches the original JSONL files in `~/.claude/projects/`. Those belong to Claude Code.

---

## `config`

Manage the configuration file.

```bash
claude-hindsight config <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `show` | Print the current config as TOML |
| `edit` | Open the config file in `$EDITOR` |
| `validate` | Check the config file for errors |
| `reset` | Overwrite config with defaults |

```bash
claude-hindsight config show
claude-hindsight config edit
claude-hindsight config validate
```

See [Configuration](/docs/configuration) for all available options.

---

## `paths`

Manage the list of directories Hindsight scans for sessions.

```bash
claude-hindsight paths <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `list` | Show all configured scan paths |
| `add <PATH>` | Add a directory to the scan list |
| `remove <PATH>` | Remove a directory from the scan list |

```bash
claude-hindsight paths list
claude-hindsight paths add ~/work/claude-projects
claude-hindsight paths remove ~/old-project
```

After adding a new path, run `init` or `reindex` to index its contents.
