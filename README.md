# Claude Hindsight

> **20/20 hindsight for your Claude Code sessions**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

Claude Hindsight is a powerful observability tool for [Claude Code](https://claude.com/code) that transforms raw JSONL transcripts into beautiful, interactive visualizations. Debug sessions, analyze costs, and understand Claude's decision-making process—all from your terminal.

---

## Features

### Execution Tree Visualization
See the complete flow of your Claude Code session—user messages, thinking blocks, tool calls, and results—rendered as a navigable list with color-coded node types.

```
  User: Fix the authentication bug
  Assistant
  Thinking (2.3s)
  Tool: Grep
  Tool: Read
  User (Tool Result)
  Success (45ms)
  Assistant
  Thinking (1.8s)
  Tool: Edit
  User (Tool Result)
  Success (89ms)
```

### Interactive Terminal UI
Fast, keyboard-driven interface with professional features:
- Full content display (no truncation)
- Node type filtering (show only specific node types)
- Vim-like navigation shortcuts
- Breadcrumb path display
- Scroll position indicators
- Split-pane view (tree + details + metadata)

### Live Monitoring
Watch your Claude Code session in real-time as it executes. See tool calls and progress—all updating live in your terminal.

### Cost Analysis
Identify expensive operations and get detailed cost breakdowns by tool type.

### Error Debugging
Jump straight to errors with full context—see what happened before, during, and after failures.

---

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/yourusername/hindsight
cd hindsight
cargo build --release

# The binary will be at target/release/hindsight
```

**Requirements:**
- Rust 1.75+
- **Nerd Font** - For proper tree icons in the TUI. Install any Nerd Font:
  - [FiraCode Nerd Font](https://github.com/ryanoasis/nerd-fonts#patched-fonts)
  - [JetBrains Mono Nerd Font](https://github.com/ryanoasis/nerd-fonts#patched-fonts)
  - Configure your terminal to use the Nerd Font

### First Run

```bash
# Initialize (discovers your Claude Code sessions)
hindsight init

# Watch your current session live
hindsight watch

# Analyze any session
hindsight show <session-id>

# Get quick stats
hindsight stats <session-id>
```

---

## Usage

### Live Monitoring

Watch your active Claude Code session in real-time:

```bash
hindsight watch
```

The TUI will update live as Claude Code executes tools and processes messages.

### Session Analysis

Analyze any session with the interactive terminal UI:

```bash
hindsight show ce3c149
```

### Browse Sessions

List all your Claude Code sessions:

```bash
hindsight list

# Filter by project
hindsight list --project my-app

# Show only sessions with errors
hindsight list --errors

# Last 10 sessions
hindsight list --last 10
```

### Cost Analysis

Analyze session costs by tool type:

```bash
hindsight costs ce3c149
```

### Error Debugging

Jump straight to errors with full context:

```bash
hindsight errors ce3c149
```

### Search Sessions

Find sessions by content, tools, or errors:

```bash
# Search by content
hindsight search "authentication bug"

# Find sessions using specific tools
hindsight search --tool Edit --tool Bash

# Find sessions with errors
hindsight search --errors
```

---

## Interactive Terminal UI

Claude Hindsight features a professional, lazygit-style terminal interface built with [ratatui](https://github.com/ratatui-org/ratatui):

### Features
- Hierarchical execution tree with expand/collapse
- Full content display (no truncation)
- Vim-like navigation shortcuts
- Node type filtering
- Split panels (tree + details + metadata)
- Breadcrumb path showing node hierarchy
- Scroll position indicators
- Real-time updates for live sessions
- Color-coded node types

### Keyboard Shortcuts

**Navigation:**
- `j/k` or `↑↓` - Navigate list or scroll details (depending on focus)
- `Ctrl+d` / `Ctrl+u` - Half-page scroll (15 lines)
- `Ctrl+f` / `Ctrl+b` - Full-page scroll (30 lines)
- `PageDown` / `PageUp` - Full-page scroll
- `g` - Jump to top
- `G` - Jump to bottom
- `Tab` - Switch focus between list and details pane

**Filtering:**
- `/` - Start node type filter
- Type node types (comma-separated): `user,assistant,tool_use`
- `Enter` - Apply filter
- `Esc` - Cancel input
- `Alt+c` - Clear active filter
- `n` - Jump to next matching node
- `N` - Jump to previous matching node

**Other:**
- `q` - Quit

### Node Types
Filter by these node types:
- `user` - User messages and tool results
- `assistant` - Assistant responses (text and tool calls)
- `tool_use` - Tool invocations
- `tool_result` - Tool execution results
- `thinking` - Extended thinking blocks
- `progress` - Progress updates
- `system` - System messages

---

## How It Works

Claude Hindsight analyzes Claude Code's JSONL transcript files located at `~/.claude/projects/`:

```
Claude Code Session
  ↓ (writes to)
~/.claude/projects/<project>/<session>.jsonl
  ↓ (parsed by)
Claude Hindsight
  ↓ (builds)
Execution Tree + Metrics
  ↓ (displayed in)
Terminal UI
```

**Key Features:**
- No setup required - auto-discovers sessions
- Fast JSONL parsing - handles large sessions efficiently
- Real-time file watching - live updates as session progresses
- Parent-child tree structure based on UUID relationships
- Offline analysis - works on historical sessions

---

## Use Cases

### Debugging
**Problem:** "My Claude Code session failed. What went wrong?"

**Solution:**
```bash
hindsight errors <session-id>
# Jump to errors, see full context, trace execution path
```

### Cost Optimization
**Problem:** "This session was expensive. What tools cost the most?"

**Solution:**
```bash
hindsight costs <session-id>
# See cost breakdown by tool type
```

### Understanding Claude Code
**Problem:** "How does Claude approach complex tasks?"

**Solution:**
```bash
hindsight show <session-id>
# View thinking blocks, see decision-making flow, trace tool execution
```

### Live Monitoring
**Problem:** "Is Claude making progress? What's it doing?"

**Solution:**
```bash
hindsight watch
# Real-time view of tool execution and session progress
```

---

## Architecture

Claude Hindsight is built with:

- **Rust** - Fast JSONL parsing, file watching, efficient processing
- **ratatui** - Professional terminal UI with keyboard navigation
- **SQLite** - Fast session indexing and caching

**Key Components:**
- **Parser** - JSONL parsing and session data extraction
- **Analyzer** - Tree building (parent-child hierarchy based on UUIDs)
- **TUI** - Interactive terminal interface with keyboard navigation
- **Storage** - Session indexing and fast lookups

---

## Roadmap

### Phase 1: Core (Complete)
- [x] JSONL parser
- [x] Session discovery
- [x] CLI commands (`init`, `list`, `stats`, `show`, `watch`, `costs`, `errors`, `search`)
- [x] Interactive terminal UI with tree visualization
- [x] Live watch mode
- [x] Error debugging
- [x] Node type filtering
- [x] Vim-like navigation
- [x] Full content display (no truncation)

### Phase 2: Enhancements (In Progress)
- [ ] Session comparison
- [ ] Export to HTML/JSON
- [ ] Full-text content search
- [ ] Performance profiling
- [ ] Cost optimization recommendations

### Phase 3: Future
- [ ] Cross-session analytics
- [ ] Historical trends
- [ ] Plugin system
- [ ] Integration with other tools

---

## Contributing

Claude Hindsight is open source and welcomes:

- Bug reports
- Feature requests
- Documentation improvements
- Code contributions

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

- **Claude Code Team** - For building an amazing tool
- **ratatui** - Professional terminal UI framework
- **Rust community** - For excellent tooling and libraries

---

## Links

- [Documentation](docs/)
- [JSONL Structure Reference](docs/JSONL-STRUCTURE.md) - Complete guide to Claude Code transcript format
- [Changelog](CHANGELOG.md)

---

Made for the Claude Code community
