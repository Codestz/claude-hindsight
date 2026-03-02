---
layout: "../../layouts/Docs.astro"
title: "Configuration — Claude Hindsight"
description: "Configure Claude Hindsight: theme, OTLP port, key bindings, scan directories, and more."
---

# Configuration

## Config file location

Hindsight stores its configuration at:

```
~/.config/hindsight/config.toml
```

On first run, a default config file is created automatically. You can edit it directly or use the `config` subcommands below.

## Managing config

### View current config

```bash
claude-hindsight config show
```

Prints the active configuration as TOML to stdout.

### Open in editor

```bash
claude-hindsight config edit
```

Opens the config file in your `$EDITOR` (falls back to `nano` if unset).

### Validate config

```bash
claude-hindsight config validate
```

Parses the config file and reports any errors without making changes. Useful after hand-editing.

### Reset to defaults

```bash
claude-hindsight config reset
```

Overwrites your config with the default values. Your session data and SQLite index are not affected.

---

## Configurable options

| Option | Default | Description |
|--------|---------|-------------|
| `web.port` | `7227` | Port for the web dashboard (`serve`) |
| `web.host` | `127.0.0.1` | Bind address for the web dashboard |
| `otel.port` | `7228` | Port for the OTLP receiver daemon |
| `tui.theme` | `"dark"` | TUI colour theme: `"dark"` or `"light"` |
| `tui.default_view` | `"list"` | TUI default view on launch: `"list"` or `"watch"` |
| `tui.keybindings.quit` | `"q"` | TUI key to quit |
| `tui.keybindings.up` | `"k"` | TUI key to move up |
| `tui.keybindings.down` | `"j"` | TUI key to move down |
| `tui.keybindings.select` | `"Enter"` | TUI key to open selected session |
| `analytics.show_cost` | `true` | Show estimated cost in the dashboard |
| `analytics.currency` | `"USD"` | Currency symbol for cost display |

**Example `config.toml`:**

```toml
[web]
port = 7227
host = "127.0.0.1"

[otel]
port = 7228

[tui]
theme = "dark"
default_view = "list"

[tui.keybindings]
quit = "q"
up   = "k"
down = "j"

[analytics]
show_cost = true
currency  = "USD"
```

---

## Scan directories

Hindsight scans specific directories to find Claude Code session files (`.jsonl`), skills, and agents. By default, it scans `~/.claude/projects/`.

### List configured paths

```bash
claude-hindsight paths list
```

### Add a path

```bash
claude-hindsight paths add ~/work/claude-projects
```

### Remove a path

```bash
claude-hindsight paths remove ~/old-project-dir
```

Paths are stored in the config file under `[scan]`:

```toml
[scan]
paths = [
  "~/.claude/projects",
  "~/work/claude-projects",
]
```

**When to add extra paths:**

- You use Claude Code in a non-standard location
- You want to combine sessions from multiple machines (via shared storage or sync)
- You have separate work and personal Claude Code setups

After adding a new path, run `claude-hindsight init` (or `reindex`) to pick up the sessions it contains.

---

## Data locations

| Path | Contents |
|------|----------|
| `~/.config/hindsight/config.toml` | Configuration file |
| `~/.local/share/claude-hindsight/index.db` | SQLite session index |
| `~/.local/share/claude-hindsight/` | All Hindsight data (safe to delete to reset) |

To find the exact paths on your system:

```bash
claude-hindsight paths list
```
