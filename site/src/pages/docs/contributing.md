---
layout: "../../layouts/Docs.astro"
title: "Contributing — Hindsight"
description: "How to set up a dev environment and contribute to Hindsight."
---

# Contributing

Contributions are welcome! This guide covers how to set up your development environment, understand the project structure, and submit a pull request.

## Prerequisites

- **Rust** 1.75+ (`rustup` recommended)
- **Node.js** 20+ and **npm** 10+ (for the web dashboard)
- **Git**

## Dev setup

### 1. Clone the repository

```bash
git clone https://github.com/Codestz/claude-hindsight
cd claude-hindsight
```

### 2. Install web dependencies

```bash
cd web && npm install && cd ..
```

### 3. Build the web dashboard

The Rust build script (`build.rs`) expects `web/out/` to exist. Run:

```bash
cd web && npm run build && cd ..
```

### 4. Build and run Hindsight

```bash
cargo run -- serve
```

Or, for faster iteration without a full rebuild of the embedded assets:

```bash
cargo run -- serve
# In another terminal:
cd web && npm run dev  # Next.js dev server on :3000
```

> **Tip:** The Next.js dev server proxies API calls to the Rust backend on `:7227`. Both can run simultaneously during development.

### 5. Run tests

```bash
cargo test
```

### 6. Lint

```bash
cargo clippy -- -D warnings
cd web && npm run lint
```

## Project structure

```
claude-hindsight/
  Cargo.toml            Rust workspace manifest
  build.rs              Embeds web/out/ at compile time
  src/                  Rust source
    main.rs
    server/
    indexer/
    tui/
    watcher.rs
  web/                  Next.js dashboard
    src/
      app/              Next.js App Router pages
      components/       React components
      lib/              API client, types, utils
  site/                 Astro docs & landing page
  tests/                Integration tests
  Formula/              Homebrew formula
```

## Code standards

### Rust

- Follow standard Rust idioms; run `cargo clippy` before committing
- Use `thiserror` for error types, `anyhow` for application-level errors
- No `unwrap()` in library code — propagate errors with `?`
- Document public items with `///` doc comments

### TypeScript / React

- Functional components only
- Minimal dependencies — avoid adding new npm packages unless necessary
- Tailwind for styling; avoid inline style objects except for dynamic values
- No `any` types

### Commit messages

Use conventional commits:

```
feat: add session export to CSV
fix: handle empty JSONL files in indexer
docs: update architecture diagram
chore: bump lucide-react to 0.470
```

## Submitting a pull request

1. Fork the repository and create a branch from `main`
2. Make your changes, following the code standards above
3. Ensure `cargo test` and `cargo clippy -- -D warnings` pass
4. Ensure `cd web && npm run build` succeeds (the rust-embed step requires a clean build)
5. Open a pull request with a clear description of the change and why it's needed

## Reporting issues

Use [GitHub Issues](https://github.com/Codestz/claude-hindsight/issues). Please include:

- Hindsight version (`hindsight --version`)
- OS and architecture
- Steps to reproduce
- Expected vs actual behavior
