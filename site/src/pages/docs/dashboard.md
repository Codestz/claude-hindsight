---
layout: "../../layouts/Docs.astro"
title: "Web Dashboard — Claude Hindsight"
description: "Complete guide to the Hindsight web dashboard — session inspector, 3D graph, image previews, cost tracking, and more."
---

# Web Dashboard

The web dashboard is the **recommended way** to use Hindsight. It provides the richest experience — two-panel session inspection, 3D graph visualization, image previews, syntax-highlighted code, and real-time streaming. While the TUI is useful for quick terminal-based browsing, the dashboard unlocks the full power of the tool.

## Starting the dashboard

```bash
# Start and open your browser
claude-hindsight serve --open

# Custom port
claude-hindsight serve --port 8080 --open
```

The dashboard is a React application embedded in the binary — no Node.js, no network requests, no build step. It runs entirely on your machine.

---

## Dashboard Home ( `/` )

The home page shows aggregate statistics across all your indexed sessions:

| Metric | What it shows |
|--------|---------------|
| **Sessions** | Total count of all indexed sessions |
| **Projects** | Number of distinct projects |
| **Total Cost** | Estimated USD cost across all sessions (from OTEL telemetry) |
| **Errors** | Total error nodes across all session transcripts |
| **Conversations/day** | Sparkline chart for the last 14 days |
| **Token Breakdown** | Input, output, cache read, cache creation totals |
| **Recent Sessions** | Last 8 sessions with first message preview |
| **Top Tools** | Most-used tools across all sessions (Bash, Read, Edit, etc.) |
| **Cost by Model** | USD breakdown by Claude model variant |

---

## Session Detail ( `/sessions/:id` )

The most powerful view in the dashboard. A **two-panel resizable layout** for inspecting any session.

### Layout

```
┌──────────────────────┬───────────────────────────┐
│   Execution List     │   Node Detail Panel       │
│   (left panel)       │   (right panel)           │
│                      │                           │
│   Click any row →    │   Full content, syntax    │
│   to inspect         │   highlighting, images    │
└──────────────────────┴───────────────────────────┘
```

- **Drag the divider** between panels to resize
- **Preset buttons** (30/50/70) snap to common ratios
- Layout persists across page reloads

### View modes

Toggle between two views using the **LIST / GRAPH** buttons:

**LIST view** — Flat chronological execution list. Each row shows:
- Color-coded left border by node type
- Type badge (user, asst, think, tool, result, task, prog)
- Tool name badge (for tool calls: Read, Edit, Bash, etc.)
- Summary text (truncated)
- File chip (if the node references a file)
- Error indicator (rose dot)
- Timestamp

**GRAPH view** — Interactive 3D force-directed graph powered by Three.js + WebGL:
- Nodes colored by type (cyan=user, green=assistant, amber=tool, etc.)
- Node size scales by importance
- Radial DAG layout with bloom post-processing
- Click a node to select it — camera flies to focus
- Drag to orbit, scroll to zoom, right-drag to pan
- Performance-scaled: low-detail mode for 1000+ node sessions

### Filter bar

Filter the execution list by node type:

| Chip | What it shows |
|------|---------------|
| **User** | Human messages (excludes tool results) |
| **Assistant** | Claude's text responses |
| **Tool** | Tool calls + tool results |
| **Task** | Sub-agent completion notifications |
| **Error** | Any node with `has_error: true` |
| **Thinking** | Extended thinking blocks |
| **Prompt** | High-scoring prompts (score ≥ 40) |

Additional controls:
- **Search** — keyword search across message content, tool names, file paths
- **Sort** — NEWEST ↓ (default) or OLDEST ↑
- **AUTO** — auto-scroll when new SSE nodes arrive
- **Node count** — shows `filtered / total` when filtering is active

### Timeline scrubber

The thin horizontal bar below the filters shows the full session timeline:
- Colored tick marks for each node (positioned by timestamp)
- Token-per-turn cost bars as background fill
- Blue playhead shows the selected node's position
- Click anywhere to jump to the nearest node
- Start/end timestamps shown inline

### Node detail panel

Click any node in the list to see its full content in the right panel:

**Tool calls** show:
- Tool name (large, amber) with MCP server badge if applicable
- Specialized rendering per tool type:
  - **Read** — file path header, line range (e.g., "lines 42–92")
  - **Edit** — diff view with removed/added lines, file stats (−12 +28)
  - **Write** — file path + full content with syntax highlighting
  - **Bash** — command in bash syntax highlighting
  - **MCP tools** — short name + server badge (e.g., `computer` · `claude-in-chrome`)
  - **Generic** — labeled field display for simple inputs, JSON for complex
- `→ RESULT` button to jump to the corresponding result node
- Token usage footer

**Tool results** show:
- Output content with automatic language detection and syntax highlighting
- File path header when available (from metadata)
- Markdown rendering when content looks like markdown
- Image preview with EXPAND lightbox for base64 screenshots
- `← CALL` button to jump back to the corresponding tool call
- Error styling (red border) for failed results

**Task notifications** show:
- Structured card with status badge (✓ COMPLETED)
- Summary, token usage, tool count, duration
- Task ID
- Result rendered as markdown

**Other node types:**
- **Thinking** — animated dots + italic text
- **Progress** — message + optional percentage bar
- **User/Assistant** — markdown-rendered content

### Raw JSON

Every node has a **RAW** toggle button in the detail panel header — shows the complete JSON structure of the node.

---

## Projects ( `/projects` )

Grid of all your project directories with:
- Session count
- Total size on disk
- Last active time
- Relative size bar (% of largest project)

Click a project to see its sessions.

---

## Activity ( `/activity` )

Real-time feed of tool calls and session events from OTEL telemetry and Claude Code hooks.

| Stat | Source |
|------|--------|
| **Tools** | Total hook tool events |
| **Agents** | Sub-agent spawn events |
| **Perms** | Permission check events |
| **Errors** | Global error count (from transcripts) |

Filter tabs: All, Tools, Agents, Lifecycle, Errors

The **Errors** panel on the right sidebar shows actual error events (PostToolUseFailure) with error messages and session links.

> **Requires hooks.** Run `claude-hindsight integrate --otel` to install hooks. See [Hooks Setup](/docs/hooks).

---

## Prompts ( `/prompts` )

Shows user prompts across all sessions, scored by quality. Prompts with score ≥ 40 appear here. Useful for reviewing what you asked Claude and how effective each prompt was.

---

## Skills ( `/skills` ) and Agents ( `/agents` )

Discovery views for Claude Code skills and agents found in your scan directories. Shows name, description, scope badges, and file paths.

Click any skill/agent to see its full definition and references.
