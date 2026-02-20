---
layout: "../../layouts/Docs.astro"
title: "CLI Commands — Claude Hindsight"
description: "Full CLI reference for all Claude Hindsight subcommands and flags."
---

# CLI Commands

Complete reference for the `claude-hindsight` command-line interface.

## Global flags

| Flag | Description |
|------|-------------|
| `--help`, `-h` | Print help for any command |
| `--version`, `-V` | Print version string |

---

## `claude-hindsight init`

Scan Claude session directories and build the SQLite index for the first time.

```bash
claude-hindsight init [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--dir <PATH>` | `~/.claude/projects` | Root directory containing project subdirectories |
| `--db <PATH>` | `~/.local/share/claude-hindsight/index.db` | Path to the SQLite database to write |

**Example:**

```bash
claude-hindsight init --dir ~/work/claude-projects
```

---

## `claude-hindsight serve`

Start the web dashboard server.

```bash
claude-hindsight serve [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `7227` | TCP port to listen on |
| `--host <HOST>` | `127.0.0.1` | Bind address |
| `--open` | false | Open browser automatically on start |

**Example:**

```bash
claude-hindsight serve --port 8080 --open
```

---

## `claude-hindsight list`

Launch the interactive terminal UI for browsing sessions.

```bash
claude-hindsight list [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--project <NAME>` | (all) | Filter to a specific project |
| `--limit <N>` | 50 | Maximum number of sessions to load |

**Keybindings in TUI:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate sessions |
| `Enter` | Open session detail |
| `q` | Quit |
| `/` | Filter sessions |

---

## `claude-hindsight show`

Print details of a single session to stdout.

```bash
claude-hindsight show <SESSION_ID>
```

Prints the session's metadata, message count, and a tree of tool calls.

**Example:**

```bash
claude-hindsight show abc12345
```

---

## `claude-hindsight watch`

Tail the most recently modified session file in real time.

```bash
claude-hindsight watch [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--session <ID>` | (latest) | Watch a specific session by ID |
| `--project <NAME>` | (all) | Restrict to a project |

Claude Hindsight follows the JSONL file from the end, printing each new node as it arrives.

**Example:**

```bash
claude-hindsight watch --project my-app
```

---

## `claude-hindsight reindex`

Incrementally update the index with new or changed session files.

```bash
claude-hindsight reindex [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--dir <PATH>` | (from init config) | Override scan directory |
| `--full` | false | Drop and rebuild the entire index |

Run this after long Claude Code sessions to pick up new files without re-scanning everything.

---

## `claude-hindsight search`

Search session content from the command line.

```bash
claude-hindsight search <QUERY> [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--project <NAME>` | (all) | Restrict search to a project |
| `--limit <N>` | 20 | Max results to return |
| `--json` | false | Output results as JSON |

**Example:**

```bash
claude-hindsight search "authentication middleware" --project api-server
```
