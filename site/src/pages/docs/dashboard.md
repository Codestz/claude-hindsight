---
layout: "../../layouts/Docs.astro"
title: "Web Dashboard — Claude Hindsight"
description: "A tour of every page in the Hindsight web portal — sessions, analytics, activity timeline, skills, and agents."
---

# Web Dashboard

The Hindsight web portal gives you a full-featured browser UI over your session data. This page walks through every section.

## Starting the dashboard

```bash
# Start (visit http://localhost:7227 manually)
claude-hindsight serve

# Start and open your browser automatically
claude-hindsight serve --open

# Start on a different port
claude-hindsight serve --port 8080 --open
```

The dashboard is a React application embedded in the binary — no Node.js, no network requests, no build step.

---

## Dashboard ( `/` )

The home page shows aggregate statistics across all your indexed sessions:

| Metric | What it shows |
|--------|---------------|
| **Total sessions** | Count of all indexed sessions |
| **Total cost** | Estimated cost in USD (based on public model pricing) |
| **Error count** | Sessions that had at least one failed tool call |
| **Activity chart** | Sparkline of session counts for the last 14 days |
| **Recent sessions** | Last 10 sessions with model, duration, and status |
| **Top tools** | Most-used tool calls across all sessions |
| **Cost by model** | Breakdown if you use multiple Claude models |

Click any session row to jump straight to its detail view.

---

## Sessions ( `/sessions` )

The sessions list shows every indexed session, newest first.

**Filtering and search:**

- **Search bar** — full-text search across user messages and tool inputs
- **Project filter** — narrow to a specific project directory
- **Errors only** — show only sessions with failed tool calls
- **Today** — sessions from the current day

Each row shows:
- Session ID (truncated)
- Project name
- Model used
- Duration
- Token counts
- Number of tool calls
- Error indicator (if any)

---

## Session Detail ( `/sessions/:id` )

The most powerful view in the dashboard. Shows the complete execution trace for one session.

### Execution tree

Every node Claude produced is shown as a tree, colour-coded by type:

| Badge | Node type | What it means |
|-------|-----------|---------------|
| `USER` | Human message | The prompt you sent |
| `ASST` | Assistant turn | Claude's full response (may contain multiple tool calls) |
| `THINK` | Thinking block | Claude's extended thinking (token count shown) |
| `TOOL` | Tool call | A tool invocation (name + input shown) |
| `RESULT` | Tool result | The output returned to Claude |
| `SUB` | Subagent | A spawned subagent session (expandable) |

### Filtering nodes

Use the type-filter chips at the top of the tree to show only the node types you care about — e.g. hide `THINK` blocks to focus on tool calls.

### Inspecting a node

Click any node to expand its full content:

- **TOOL** — shows the full input object sent to the tool
- **RESULT** — shows the complete output (with diff view for Edit calls)
- **THINK** — shows the raw thinking text
- **USER/ASST** — shows the full message

### Raw JSON

Every node has a "Raw JSON" toggle at the bottom of its detail pane — useful for debugging or when you want the exact data structure Claude received.

---

## Projects ( `/projects` )

Aggregated view across all your project directories.

For each project:

- Session count
- Total token usage
- Total estimated cost
- Error rate
- Most-used model
- Last active date

Click a project to filter the Sessions list to that project.

---

## Activity Timeline ( `/activity` )

Real-time feed of tool calls and session lifecycle events, powered by OTLP telemetry.

> **Requires hooks.** The Activity Timeline is only populated when you have `PreToolUse` and `UserPromptSubmit` hooks installed. See [Hooks Setup](/docs/hooks).

The timeline shows:

- **`HOOK — session:start`** — a new Claude Code session began
- **`OTLP — tool:*`** — a tool call fired (name shown after the colon)
- **`HOOK — session:end`** — the session completed

Events stream in real time while a session is running. Historical events from previous sessions are shown below the live stream.

Each event shows its timestamp and the hook or OTLP source.

---

## Skills ( `/skills` )

Shows all skill files (`.md` files with `description:` frontmatter) discovered in your scan directories.

For each skill:

- File name and path
- Description (from frontmatter)
- Trigger description

This page is useful for auditing which skills Claude Code can access in each project.

Skill discovery requires that your project directories are added to Hindsight's scan paths. See [Configuration → Scan directories](/docs/configuration#scan-directories).

---

## Agents ( `/agents` )

Shows all agent definitions discovered in your scan directories — typically JSON or YAML files that define custom Claude Code agents.

For each agent:

- Agent name
- Source file path
- Description
- Capabilities listed in the definition

Click an agent to see its full definition.

Like skills, agent discovery uses the configured scan paths.
