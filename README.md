# 🔮 Claude Hindsight

> **20/20 hindsight for your Claude Code sessions**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

Claude Hindsight is a powerful observability tool for [Claude Code](https://claude.com/code) that transforms raw JSONL transcripts into beautiful, interactive visualizations. Debug sessions, optimize costs, and understand Claude's decision-making process—all from your terminal or browser.

---

## ✨ Features

### 🌲 **Execution Tree Visualization**
See the complete hierarchy of your Claude Code session—user messages, thinking blocks, tool calls, and subagent spawns—rendered as a beautiful, navigable tree.

```
📨 User: "Fix the authentication bug"
│
├─ 🧠 Thinking (2.3s)
│  │ I need to understand the current auth implementation...
│
├─ 🔍 Tool: Grep (*.ts) - 123ms ✓
│  │ Found 3 files
│
├─ 📖 Tool: Read (src/auth.ts) - 45ms ✓
│
├─ 🧠 Thinking (1.8s)
│  │ I see the issue - missing token validation...
│
└─ ✏️ Tool: Edit (src/auth.ts) - 89ms ✓
```

### ⚡ **Live Monitoring**
Watch your Claude Code session in real-time as it executes. See tool calls, costs, and progress—all updating live in your terminal.

### 💰 **Cost Optimization**
Identify expensive operations, detect repeated tool calls, and get AI-powered suggestions to reduce costs and improve performance.

### 🐛 **Smart Debugging**
Jump straight to errors with full context—see what happened before, during, and after failures.

### 🎯 **Two Interfaces, One Tool**
- **Terminal UI** (default) - Fast, keyboard-driven, works over SSH
- **Web Dashboard** (optional) - Rich visualizations, charts, and sharing

---

## 🚀 Quick Start

### Installation

```bash
# macOS (Homebrew)
brew install hindsight

# From source
git clone https://github.com/yourusername/hindsight
cd hindsight
cargo install --path .
```

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

## 📖 Usage

### Live Monitoring

Watch your active Claude Code session in real-time:

```bash
hindsight watch
```

**Output:**
```
📊 Watching: my-project/ce3c149... (live)

Duration: 00:45:23
Tools: 128 (+3 in last 10s)
Cost: $2.15

Currently executing:
  ▶ Tool: Bash - npm run build (3.2s)

[d] Dashboard | [q] Quit | [/] Search
```

### Session Analysis

Analyze any session with an interactive terminal UI:

```bash
hindsight show ce3c149
```

Or open the web dashboard for rich visualizations:

```bash
hindsight show ce3c149 --dashboard
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

Find expensive operations and optimization opportunities:

```bash
hindsight costs ce3c149

💰 Cost Analysis

Total: $2.15

By Tool:
  Task (subagents)  $0.72 (33%)  █████████████
  Bash              $0.45 (21%)  ████████
  Read              $0.38 (18%)  ███████

Optimization Opportunities:
  ⚠️ Read called 8x on same file → Save $0.15
  ⚠️ 3 failed Edit operations → Wasted $0.08
```

### Error Debugging

Jump straight to errors with full context:

```bash
hindsight errors ce3c149

🚨 Error Analysis

Found 3 errors:

Error #1 (00:23:45)
  Tool: Read
  File: src/missing.txt
  Error: File not found

  Context:
    User → Grep (*.txt) → Read (failed)
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

## 🎨 Interactive Terminal UI

Claude Hindsight features a beautiful, `lazygit`-style terminal interface built with [ratatui](https://github.com/ratatui-org/ratatui):

**Features:**
- ✅ Collapsible execution tree (expand/collapse with Enter)
- ✅ Keyboard navigation (vim-style: j/k or arrows)
- ✅ Split panels (tree view + details)
- ✅ Real-time updates (for live sessions)
- ✅ Syntax-highlighted JSON viewer
- ✅ Search and filter (/ to search)
- ✅ Color-coded nodes (success=green, error=red, thinking=blue)

**Keyboard Shortcuts:**
- `↑↓` or `j/k` - Navigate tree
- `Enter` or `Space` - Expand/collapse node
- `i` - View tool input
- `o` - View tool output
- `/` - Search
- `d` - Open dashboard
- `q` - Quit

---

## 🌐 Web Dashboard

For deep analysis and sharing, launch the web dashboard:

```bash
hindsight show ce3c149 --dashboard
```

**Features:**
- 📊 Interactive execution tree with zoom/pan
- 📈 Cost breakdown charts (Recharts)
- ⏱️ Timeline view with parallel execution
- 🔍 Full-text search across session
- 📤 Export to HTML report
- 🎨 Beautiful, modern UI (React + Tailwind)

---

## 🏗️ How It Works

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
Terminal UI or Web Dashboard
```

**Key Features:**
- ✅ No setup required - auto-discovers sessions
- ✅ Zero-copy parsing - fast analysis even for large sessions
- ✅ Real-time file watching - live updates as session progresses
- ✅ Subagent tracking - understands multi-agent workflows
- ✅ Offline analysis - works on historical sessions

---

## 📊 Use Cases

### 🐛 Debugging
**Problem:** "My Claude Code session failed. What went wrong?"

**Solution:**
```bash
hindsight show --current
# Jump to errors, see full context, trace execution path
```

### 💰 Cost Optimization
**Problem:** "This session cost $10. Why?"

**Solution:**
```bash
hindsight costs <session-id>
# See expensive tools, detect redundant operations, get optimization tips
```

### 🔍 Understanding Claude Code
**Problem:** "How does Claude approach complex tasks?"

**Solution:**
```bash
hindsight show <session-id>
# View thinking blocks, see decision-making flow, understand agent orchestration
```

### ⚡ Live Monitoring
**Problem:** "Is Claude stuck? What's it doing?"

**Solution:**
```bash
hindsight watch
# Real-time view of current tool, progress, and costs
```

---

## 🛠️ Architecture

Claude Hindsight is built with:

- **Rust** - Fast JSONL parsing, file watching, zero-copy processing
- **ratatui** - Beautiful terminal UI with keyboard navigation
- **React + TypeScript** - Rich web dashboard (optional)
- **SQLite** - Fast session indexing and caching

**Key Components:**
- 📝 **JSONL Parser** - Parses Claude Code transcripts
- 🌲 **Tree Builder** - Constructs hierarchical execution tree
- 💾 **Session Index** - Fast lookups across all sessions
- 🖥️ **Terminal UI** - Interactive ratatui-based interface
- 🌐 **Web Server** - Optional dashboard (embedded in binary)

---

## 🗺️ Roadmap

### ✅ Phase 1: Core (Weeks 1-2)
- [x] JSONL parser
- [x] Session discovery
- [x] CLI commands (`init`, `list`, `stats`)
- [ ] Interactive terminal UI
- [ ] Live watch mode
- [ ] Error debugging

### 🚧 Phase 2: Advanced (Weeks 3-4)
- [ ] Web dashboard
- [ ] Cost analysis
- [ ] Search functionality
- [ ] Session comparison
- [ ] Export to HTML

### 🔮 Phase 3: Future
- [ ] OpenTelemetry integration (optional)
- [ ] Cross-session analytics
- [ ] Team dashboards
- [ ] Cost alerting
- [ ] Plugin system

---

## 🤝 Contributing

We love contributions! Claude Hindsight is open source and welcomes:

- 🐛 Bug reports
- 💡 Feature requests
- 📝 Documentation improvements
- 🔧 Code contributions

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good First Issues:**
- [Add syntax highlighting to terminal JSON viewer](issues/1)
- [Support filtering sessions by date range](issues/2)
- [Add export to CSV format](issues/3)

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- **Claude Code Team** - For building an amazing tool
- **ratatui** - Beautiful terminal UI framework
- **LangSmith** - Inspiration for observability UX

---

## 🔗 Links

- [Documentation](docs/)
- [Changelog](CHANGELOG.md)
- [Issues](https://github.com/yourusername/hindsight/issues)
- [Discussions](https://github.com/yourusername/hindsight/discussions)

---

<p align="center">
  Made with ❤️ for the Claude Code community
</p>

<p align="center">
  <strong>⭐ Star us on GitHub if Claude Hindsight helps you!</strong>
</p>
