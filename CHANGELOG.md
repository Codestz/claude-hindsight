# Changelog

All notable changes to claude-hindsight are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). This project uses [Semantic Versioning](https://semver.org/).

---

## [2.3.0] — 2026-03-15

Major redesign — new two-panel session inspector, 3D graph visualization, clean architecture refactor, and smart update detection.

### Added
- **Two-panel session inspector** — resizable split layout with flat execution list (left) and full node detail panel (right), replacing the chat-style timeline
- **3D execution graph** — interactive force-directed visualization via react-force-graph-3d with bloom post-processing, performance tiers, and click-to-inspect
- **Image previews** — inline base64 image rendering with expandable lightbox from session transcripts
- **Task notification cards** — parsed `<task-notification>` XML from sub-agent completions into structured display
- **Custom syntax highlighting** — built-in tokenizer supporting TypeScript, JavaScript, Rust, Python, Go, Bash, HTML, and CSS with no external dependencies
- **Timeline scrubber** — visual scrubber with token-per-turn cost overlay across session turns
- **Update detection** — automatically detects binary upgrades and suggests `reindex`; checks GitHub releases once/day for newer versions
- **Reindex project name fix** — `reindex` now corrects stale project names by re-deriving from disk paths
- **Token breakdown** — session header shows Input, Output, Cache Read, and Cache Write token counts separately
- **MCP tool name parsing** — `mcp__server__tool` names render as short tool name with server badge

### Changed
- **Clean architecture refactor** — every component directory has `types.ts`, `config.ts`, `utils.ts`, and barrel exports; one component per file per export; zero inline interfaces
- **Hooks extracted** — `useSessionData`, `useNodeFiltering`, `useResizableRatio`, `useGraphSetup`, `useNodeContent` replace inline logic in page components
- **Tool displays split** — 469-line monolith split into `primitives.tsx`, `tool-renderers.tsx`, `serena.tsx`, `strip-utils.ts`, `resolve-file-path.ts`
- **CodeRender** — 303 → 96 lines, imports from new `lib/syntax/` module
- **NodeDetailPanel** — 919 → 221 lines via `useNodeContent` hook extraction
- **SessionDetail** — 245 → 102 lines, pure composition with hooks
- **Duration formatting** — shows hours (`10h 42m`) instead of raw minutes (`642m`)
- **Dashboard/Activity** — all `useMemo` calls moved before early returns (Rules of Hooks compliance)
- **Project name decoding** — rewrote `decode_project_name` to handle encoded directory paths correctly

### Fixed
- Tool results showing as TEXT instead of correct language — now checks 6 file path sources
- Line number stripping regex handles leading whitespace
- Stack overflow in TimelineScrubber with large arrays (loop-based min/max)
- XSS in graph tooltips (HTML entity escaping)
- SVG execution in image previews (restricted to safe types + size limit)
- Clippy warnings (12 fixes: unused imports, variables, unnecessary conversions)
- Integrate command: replaced panicking `unwrap()` with safe `let-else` guards

### Removed
- Chat-style timeline components: `UserBubble`, `AssistantMessage`, `ToolCallCard`, `ToolResultInline`, `ConversationTimeline`, `NodeDetailDrawer`
- Tree view (replaced by flat execution list + 3D graph)

---

## [2.1.0] — 2026-03-08

### Added
- **Agent/skill grouping** — agents and skills now grouped by scope (project, global, user) across the dashboard
- **Subagent model pills** — session header shows which models subagents used

### Changed
- Improved session discovery performance with parallel directory scanning

---

## [2.0.0] — 2026-02-28

Major release — full OpenTelemetry integration, Claude Code hooks, background daemon, and a complete web portal redesign.

### Added
- **OTLP telemetry** — built-in OpenTelemetry log receiver (`/v1/logs` endpoint) for ingesting Claude Code telemetry spans in real time
- **Claude Code hooks** — `UserPromptSubmit` and `PreToolUse` hooks with `claude-hindsight hook` command; auto-spawns background daemon on every session
- **Background daemon** — `claude-hindsight daemon` runs an OTLP-capable server in the background; `serve` kills any existing daemon before starting
- **Agent discovery** — new `agents` module with automatic skill/agent discovery from project directories, supporting subdirectory layouts and CSV frontmatter parsing
- **New CLI commands** — `daemon`, `hook`, `integrate`, `clean` for managing hooks, OTLP integration, and database cleanup
- **Activity timeline** — real-time event timeline page with OTLP-sourced activity data
- **Skills & Agents pages** — dedicated pages for browsing discovered skills and agents with detail views
- **Token breakdown chart** — per-session token usage visualization component
- **Configurable OTLP port** — server port is now configurable via settings
- **JSONL activity backfill** — existing JSONL session data can be backfilled into the OTLP store

### Changed
- **Web portal rewrite** — migrated from Next.js to Vite + React Router; full page redesign with sidebar navigation replacing top nav
- **Dashboard redesign** — new dashboard with OTEL telemetry integration, activity charts, and richer analytics
- **Storage layer** — significantly expanded SQLite schema with squashed migrations, extracted helpers, and deduplicated queries
- **Server architecture** — modular route system with dedicated route files for agents, hooks, OTLP, telemetry, and events

### Fixed
- OTLP parser robustness — handles string-encoded ints and event name fallback
- Agent discovery supports subdirectory layouts and CSV frontmatter
- `serve` command kills existing background daemon before starting
- All clippy warnings resolved

---

## [1.0.2] — 2026-02-21

### Added
- **Windows build** — release workflow now produces `x86_64-pc-windows-msvc` `.zip` binaries alongside existing macOS and Linux targets
- **Repo health** — issue templates (bug report, feature request), `SECURITY.md`, `CODE_OF_CONDUCT.md`, `FUNDING.yml`, Dependabot config, labels sync workflow, release drafter
- **Dynamic site versioning** — public docs site reads version from `Cargo.toml` at build time; no more manual version bumps in HTML

### Changed
- **Site** — nav, hero, and install sections are now fully responsive on mobile: hamburger menu, docs sidebar drawer, single-column mockup, stacked hero actions
- **PR template** — simplified to essentials (what/why, type, how to test, checklist)

### Dependencies
- `ratatui` 0.28 → 0.30 (required to align with `ratatui-core` used by tui-tree-widget 0.24)
- `tui-tree-widget` 0.22 → 0.24
- `axum` 0.7 → 0.8
- `tower` 0.4 → 0.5, `tower-http` 0.5 → 0.6
- `rusqlite` 0.32 → 0.38
- `notify` 6.1 → 8.2
- `dirs` 5.0 → 6.0
- `toml` 0.8 → 1.0
- `next` 15 → 16, `tailwindcss` 3 → 4 (v4 migration), `eslint` 9 → 10
- GitHub Actions: `actions/checkout` 4 → 6, `actions/cache` 4 → 5, `actions/setup-node` 4 → 6

---

## [1.0.1] — 2026-02-20

### Changed
- Renamed binary and crate from `hindsight` to `claude-hindsight` for clarity
- Renamed Homebrew formula to match new binary name with real SHA256 hashes

### Added
- SEO meta tags, Open Graph image, favicon, and sitemap for the public site
- `workflow_dispatch` trigger on the Pages deployment workflow

---

## [1.0.0] — 2026-02-19

First stable release.

### Added
- **TUI** — interactive terminal interface built with ratatui: project/session browser, conversation tree, node detail panel, vim-style navigation, FTS5 search, diff view, raw JSON view, replay, and clipboard copy
- **Web dashboard** — embedded Next.js dashboard served via `claude-hindsight serve`; Cyber-Industrial aesthetic, node detail panel, path-based routing, subagent model display
- **CLI commands** — `list`, `show`, `search`, `watch`, `reindex`, `paths`, `serve`
- **Storage** — SQLite-backed session indexing with FTS5 full-text search
- **File watcher** — live session ingestion via `notify` (macOS kqueue)
- **Analytics** — per-session stats, subagent cost aggregation, error detection
- **Configuration** — TOML config file with CLI and TUI integration; configurable Claude project directories
- **Distribution** — pre-built binaries for macOS (ARM + x86), Linux (ARM + x86); Homebrew formula; crates.io publish

---

<!-- next release notes will be drafted automatically by release-drafter -->
