# Changelog

All notable changes to claude-hindsight are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). This project uses [Semantic Versioning](https://semver.org/).

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
