# Changelog

All notable changes to claude-hindsight are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). This project uses [Semantic Versioning](https://semver.org/).

---

## [1.0.1] — 2025-01-XX

### Changed
- Renamed binary and crate from `hindsight` to `claude-hindsight` for clarity
- Renamed Homebrew formula to match new binary name with real SHA256 hashes

### Added
- SEO meta tags, Open Graph image, favicon, and sitemap for the public site
- `workflow_dispatch` trigger on the Pages deployment workflow

---

## [1.0.0] — 2025-01-XX

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
