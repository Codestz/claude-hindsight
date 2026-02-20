# Contributing to Claude Hindsight

Thank you for your interest in contributing! This document covers setup, standards, and the PR process.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)

---

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR-USERNAME/claude-hindsight.git
   cd claude-hindsight
   ```
3. Set up your development environment (see below)
4. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

---

## Development Setup

### Prerequisites

- **Rust** 1.75+ ([rustup.rs](https://rustup.rs/))
- **Node.js** 20+ ([nodejs.org](https://nodejs.org/)) — for the web dashboard
- **Git**
- **Claude Code** — for testing against real sessions

### Backend (Rust)

```bash
cargo build           # debug build (uses web/out/ from disk if present)
cargo build --release # release build (embeds web/out/ into binary)
cargo test            # run all tests
cargo clippy -- -D warnings
cargo fmt
```

### Frontend (Next.js)

```bash
cd web
npm install
npm run dev    # dev server on http://localhost:3000
npm run build  # static export to web/out/
npm run lint
```

### Full build (what CI does)

```bash
make build     # npm run build → cargo build --release
```

### Running in development

Run both servers separately — the Rust server serves the API, Next.js serves the UI with hot reload:

```bash
# Terminal 1
make dev-rust          # cargo run -- serve --port 7227

# Terminal 2
make dev-web           # cd web && npm run dev (port 3000)
```

Point your browser at `http://localhost:3000`. The Next.js dev server proxies `/api/*` to the Rust backend.

---

## Project Structure

```
claude-hindsight/
├── src/
│   ├── main.rs                   # CLI entry point
│   ├── lib.rs                    # library root
│   ├── error.rs                  # shared error types
│   ├── commands/                 # CLI subcommands
│   │   ├── mod.rs
│   │   ├── init.rs               # discover sessions
│   │   ├── list.rs               # list sessions
│   │   ├── show.rs               # TUI session viewer
│   │   ├── watch.rs              # live session monitor
│   │   ├── serve.rs              # web server launcher
│   │   ├── reindex.rs            # rebuild SQLite index
│   │   ├── search.rs
│   │   ├── export.rs
│   │   └── config.rs
│   ├── parser/                   # JSONL parsing
│   │   ├── mod.rs
│   │   ├── transcript.rs
│   │   └── models.rs             # ExecutionNode and related types
│   ├── storage/                  # session discovery + SQLite index
│   │   ├── mod.rs
│   │   ├── index.rs
│   │   └── discovery.rs
│   ├── analyzer/                 # tree building + analytics
│   │   ├── mod.rs
│   │   ├── tree.rs
│   │   ├── simple_tree.rs
│   │   ├── session_analytics.rs
│   │   └── smart_label.rs
│   ├── server/                   # axum HTTP server
│   │   ├── mod.rs                # router + embedded asset handler
│   │   ├── dto.rs                # request/response types
│   │   ├── error.rs
│   │   └── routes/
│   │       ├── analytics.rs
│   │       ├── events.rs         # SSE live feed
│   │       ├── projects.rs
│   │       ├── search.rs
│   │       └── sessions.rs
│   ├── tui/                      # ratatui terminal UI
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── router.rs
│   │   ├── render.rs
│   │   ├── ui.rs
│   │   ├── events.rs
│   │   ├── theme.rs
│   │   ├── dashboard_view.rs
│   │   ├── sessions_view.rs
│   │   ├── projects_view.rs
│   │   ├── search_modal.rs
│   │   ├── search.rs
│   │   └── code_render.rs
│   ├── watcher/                  # file watching + streaming
│   │   ├── mod.rs
│   │   └── stream_renderer.rs
│   ├── api/                      # shared response formatting
│   │   ├── mod.rs
│   │   ├── responses.rs
│   │   └── formatting.rs
│   ├── search/                   # search engine
│   │   ├── mod.rs
│   │   ├── filters.rs
│   │   └── results.rs
│   └── config/
│       └── mod.rs
├── web/                          # Next.js 15 frontend
│   ├── src/
│   │   ├── app/                  # Next.js App Router pages
│   │   ├── components/           # React components
│   │   └── lib/                  # API client, types, utils
│   ├── package.json
│   └── next.config.ts
├── build.rs                      # auto-builds frontend before rustc
├── Cargo.toml
├── Makefile
└── README.md
```

---

## Coding Standards

### Rust

We follow the [Apollo GraphQL Rust Best Practices Handbook](https://github.com/apollographql/rust-best-practices). Use the `/rust-best-practices` skill when writing or reviewing Rust code.

**Key principles:**
- Borrow over clone — prefer `&str`, `&Path`, `as_deref()` over `.clone()`
- Use `thiserror` for errors; avoid `unwrap()` in production paths
- Propagate errors with `?`; no panics outside tests
- Prefer iterator combinators over manual loops
- Run `cargo clippy -- -D warnings` before committing — CI enforces it

**Before committing:**
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

### Web (Next.js / React)

We follow [Vercel React Best Practices](https://github.com/vercel-labs/react-best-practices). Use the `/vercel-react-best-practices`, `/vercel-composition-patterns`, `/frontend-design`, and `/web-design-guidelines` skills when working on the frontend.

**Key principles:**
- Direct imports, no barrel files — keeps bundles small
- Compound components over boolean props
- `React.memo` for expensive renders; functional `setState` for correctness
- Distinctive design — avoid generic AI design patterns

**Before committing:**
```bash
npm run build    # must succeed cleanly
npm run lint
```

---

## Testing

### Rust unit tests

```bash
cargo test                        # run all tests
cargo test watcher                # run tests matching "watcher"
cargo test -- --nocapture         # show println! output
```

**Writing tests:**
- Descriptive names: `test_read_new_nodes_returns_only_appended_lines()`
- Use `tempfile::NamedTempFile` for file-based tests
- One assertion per logical behavior

### Integration tests

```bash
cargo test --test '*'             # run tests/ directory
```

Place integration tests in `tests/` with real-world JSONL fixtures in `tests/fixtures/`.

---

## Pull Request Process

1. **Run all checks** before pushing:
   ```bash
   cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
   cd web && npm run build && npm run lint
   ```
2. **Write a clear PR description** — what changed and why
3. **Link related issues** with `Closes #N`
4. **Add tests** for new behaviour
5. **Update this README** if commands or architecture change
6. Address review feedback; squash commits if asked before merge

### Commit message format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`

**Examples:**
```
feat(serve): add --open flag to launch browser on start
fix(watcher): start position at end of file instead of beginning
perf(storage): add index on session_id column
```

---

## Areas for Contribution

### Good first issues

- [ ] Add keyboard shortcut help screen in TUI (press `?`)
- [ ] Add syntax highlighting to JSON viewer in terminal UI
- [ ] Support filtering `hindsight list` by date range (`--since`, `--until`)
- [ ] Add `--json` output flag to `hindsight list` for scripting
- [ ] Create example JSONL fixtures for integration tests
- [ ] Improve error messages with actionable suggestions

### Medium difficulty

- [ ] Session comparison view (diff two sessions side-by-side)
- [ ] Cost breakdown chart in the web dashboard
- [ ] Export session to HTML report
- [ ] Add pagination to the TUI execution tree for very large sessions
- [ ] Dark/light theme toggle in the web dashboard

### Advanced

- [ ] Pattern detection — flag repeated tool failures, excessive retries
- [ ] Cross-session analytics — trends over time, per-project cost history
- [ ] Session replay mode — step through execution with timeline scrubber
- [ ] OpenTelemetry export (optional integration)

---

## Questions?

- Open a GitHub issue for bugs or feature requests
- Start a discussion for broader design questions
- Check the README for architecture and command reference

## Code of Conduct

Be respectful, provide constructive feedback, assume positive intent, and help others learn and grow.

## License

By contributing, you agree your contributions will be licensed under the MIT License.

---

Thank you for contributing to Claude Hindsight!
