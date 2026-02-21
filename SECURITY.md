# Security Policy

## Supported Versions

Only the latest released version of claude-hindsight receives fixes.

| Version | Supported |
| ------- | --------- |
| 1.x     | ✅        |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for potential security problems.

Instead, use one of these private channels:

1. **GitHub private advisory** (preferred): [Report a vulnerability](https://github.com/Codestz/claude-hindsight/security/advisories/new)
2. **Email**: est.estrada@outlook.com — use the subject line `[claude-hindsight] Security`

### What to include

- A description of the issue and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected versions if known

### What to expect

- Acknowledgement within **72 hours**
- A fix or mitigation plan within **14 days** for confirmed issues
- Credit in the release notes if you'd like it

## Scope

claude-hindsight is a local CLI tool that reads Claude Code session files from your filesystem. It does not transmit data externally and does not require network access for its core functionality. Issues most relevant to this project include:

- Path traversal or arbitrary file read beyond intended directories
- SQL injection in the local SQLite queries
- Command injection via user-supplied config values
- Unintended data exposure through the local web server (`serve` command)
