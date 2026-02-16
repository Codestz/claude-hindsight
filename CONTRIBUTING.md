# Contributing to Claude Hindsight 🔭

Thank you for your interest in contributing to Claude Hindsight! This document provides guidelines and best practices for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/observatory.git
   cd observatory
   ```
3. **Set up development environment** (see below)
4. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- **Rust** 1.75+ ([Install Rust](https://rustup.rs/))
- **Git**
- **Claude Code** (for testing)

### Build Project

```bash
# Clone repository
git clone https://github.com/yourusername/hindsight
cd hindsight

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- init
cargo run -- list
cargo run -- watch
```

## Project Structure

```
hindsight/
├── src/                      # Rust source code
│   ├── main.rs              # CLI entry point
│   ├── parser/              # JSONL parser
│   │   ├── mod.rs
│   │   ├── transcript.rs    # Parse transcript files
│   │   └── models.rs        # Data structures
│   ├── storage/             # Session indexing
│   │   ├── mod.rs
│   │   ├── index.rs         # Session index
│   │   └── cache.rs         # Caching layer
│   ├── analysis/            # Analysis engine
│   │   ├── mod.rs
│   │   ├── tree.rs          # Execution tree builder
│   │   ├── costs.rs         # Cost calculation
│   │   └── insights.rs      # Pattern detection
│   ├── ui/                  # Terminal UI (ratatui)
│   │   ├── mod.rs
│   │   ├── tree_view.rs     # Tree widget
│   │   ├── details.rs       # Detail panels
│   │   └── live.rs          # Live monitoring
│   ├── commands/            # CLI commands
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── watch.rs
│   │   ├── show.rs
│   │   ├── list.rs
│   │   └── stats.rs
│   └── dashboard/           # Web dashboard (Phase 2)
│       └── mod.rs
├── tests/                   # Integration tests
├── examples/                # Example usage
├── docs/                    # Documentation
├── Cargo.toml
└── README.md
```

## Coding Standards

### Rust

We follow the [Apollo GraphQL Rust Best Practices Handbook](https://github.com/apollographql/rust-best-practices).

**Reference:** Use `/rust-best-practices` skill when writing or reviewing Rust code.

**Key principles**:
- **Borrowing over cloning**: Use `&str`, `&Path`, `as_deref()` instead of unnecessary `.clone()`
- **Proper error handling**: Use `thiserror`, avoid `unwrap()` in production code
- **No panics**: Use `Result<T, E>` and `?` operator for error propagation
- **Iterator patterns**: Prefer `.iter()` and combinators over manual loops
- **Clean up**: Use `Drop` trait for automatic resource cleanup
- **Performance**: Always benchmark with `--release`, run `cargo clippy -- -D clippy::perf`

**Before committing**:
```bash
cargo fmt              # Format code
cargo clippy --all-targets -- -D warnings  # Lint
cargo test             # Run tests
```

### Terminal UI (ratatui)

We use [ratatui](https://github.com/ratatui-org/ratatui) for the terminal UI.

**Key principles**:
- **Responsive rendering**: 60 FPS, smooth updates
- **Keyboard-first**: Vim-style navigation (j/k, gg/G)
- **State management**: Clean separation of UI state and data
- **Accessibility**: Clear visual hierarchy, color-coded nodes
- **Performance**: Lazy rendering for large trees

**Before committing**:
```bash
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo test
```

---

### React & Web Dashboard (Phase 2)

We follow [Vercel React Best Practices](https://github.com/vercel-labs/react-best-practices) and [Composition Patterns](https://github.com/vercel-labs/composition-patterns).

**Reference:** Use `/vercel-react-best-practices`, `/vercel-composition-patterns`, `/frontend-design`, and `/web-design-guidelines` skills when building the web dashboard.

**Key principles**:
- **Bundle optimization**: Direct imports (no barrel files), dynamic imports for heavy components
- **Server-side performance**: React.cache() for deduplication, minimize client serialization
- **Re-render optimization**: memo for expensive components, functional setState
- **Composition over boolean props**: Avoid boolean prop proliferation, use compound components
- **Distinctive design**: Bold aesthetic choices, avoid generic AI design patterns

**Before committing**:
```bash
npm run typecheck      # Type checking
npm run build          # Verify build works
npm run lint           # ESLint
```

### Code Review Checklist

- [ ] Code follows project conventions
- [ ] Tests added for new functionality
- [ ] Documentation updated (README, code comments)
- [ ] No console.log in production code
- [ ] Error handling is comprehensive
- [ ] Performance considered (avoid N+1 queries, unnecessary re-renders)

## Testing

### Unit Tests

```bash
cargo test                    # Run all tests
cargo test --                 # Run with output
cargo test test_name          # Run specific test
cargo test -- --nocapture     # Show print! output
```

**Writing tests**:
- Use descriptive names: `test_parse_tool_use_creates_correct_node()`
- One assertion per test when possible
- Use test fixtures in `tests/fixtures/` directory
- Clean up test files in `Drop` implementations

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_message() {
        let json = r#"{"type": "user", "message": {"content": "Hello"}}"#;
        let node = parse_line(json).unwrap();
        assert_eq!(node.node_type, NodeType::UserMessage);
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/session_analysis.rs
use hindsight::parser::parse_session;
use std::path::Path;

#[test]
fn test_parse_real_session() {
    let session = parse_session(Path::new("tests/fixtures/session.jsonl")).unwrap();
    assert_eq!(session.total_tools, 128);
}
```

## Pull Request Process

1. **Update documentation**: README, code comments, CHANGELOG
2. **Run all checks**:
   ```bash
   cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
   ```
3. **Write clear commit messages**:
   ```
   feat(collector): add support for token usage tracking
   
   - Parse token_usage field from transcript
   - Store in execution_tree table
   - Add tests for token parsing
   
   Closes #42
   ```
4. **Create pull request** with:
   - Clear description of changes
   - Screenshots for UI changes
   - Link to related issues
5. **Address review feedback** promptly
6. **Squash commits** if requested before merge

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance

## Areas for Contribution

### Good First Issues

- [ ] Add syntax highlighting to JSON viewer in terminal UI
- [ ] Support filtering sessions by date range
- [ ] Add export to CSV format
- [ ] Improve error messages with suggestions
- [ ] Add progress bars for parsing large sessions
- [ ] Create example JSONL fixtures for testing

### Medium Difficulty

- [ ] Implement session search functionality
- [ ] Add keyboard shortcuts help screen (press `?`)
- [ ] Create session comparison view
- [ ] Add cost breakdown visualization in terminal
- [ ] Implement file watcher for live mode
- [ ] Add pagination for large execution trees

### Advanced Features

- [ ] Web dashboard (Phase 2)
- [ ] OpenTelemetry integration (optional)
- [ ] Pattern detection and insights engine
- [ ] Export to HTML/PDF reports
- [ ] Session replay mode (step through execution)
- [ ] Cross-session analytics and trends

## Questions?

- Open an issue for discussion
- Join discussions in existing issues/PRs
- Check the README for architecture details

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help others learn and grow
- Assume positive intent

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Claude Hindsight! 🔭✨
