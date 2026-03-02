---
layout: "../../layouts/Docs.astro"
title: "Troubleshooting — Claude Hindsight"
description: "Solutions for common problems with Claude Hindsight: database errors, port conflicts, blank dashboard, and more."
---

# Troubleshooting

## Database errors after upgrading

**Symptom:** After upgrading to v2.0+, you see an error like:

```
Error: database schema version mismatch (found v1, expected v2)
```

**Solution:** Hindsight v2.0.1+ auto-detects schema mismatches and rebuilds the database automatically on startup. If you see this on an older v2.0.0 build, run:

```bash
claude-hindsight clean --db
claude-hindsight init
```

`clean --db` removes only the SQLite index — your original JSONL session files are never modified. `init` rebuilds the index from scratch.

---

## Port already in use

**Web dashboard (`:7227`):**

```
Error: address already in use (os error 98)
```

Either another Hindsight instance is running, or another process has claimed port 7227. Options:

```bash
# Use a different port
claude-hindsight serve --port 8080

# Or find and kill the conflicting process
lsof -i :7227
kill <PID>
```

**OTLP daemon (`:7228`):**

Same error for the daemon port. Use `--port` to override:

```bash
claude-hindsight daemon --port 7229
# Then update config so the hooks point to the new port
claude-hindsight config edit   # set otel.port = 7229
claude-hindsight integrate --otel --force  # reinstall hooks with new port
```

---

## No sessions found

**Symptom:** `claude-hindsight init` reports "Indexed 0 sessions".

**Common causes:**

1. **Wrong scan directory** — Hindsight looks in `~/.claude/projects/` by default. If your Claude Code data is elsewhere:

   ```bash
   claude-hindsight init --dir /path/to/your/claude/projects
   # Or add it permanently:
   claude-hindsight paths add /path/to/your/claude/projects
   ```

2. **No sessions yet** — Claude Code only creates JSONL files after you've used it. Start a session, then run `init` again.

3. **Empty project directories** — Claude Code creates subdirectories per project. Check that `~/.claude/projects/` contains subdirectories with `.jsonl` files:

   ```bash
   ls ~/.claude/projects/
   ```

---

## Corrupt or locked database

**Symptom:** Errors mentioning `SQLITE_CORRUPT`, `database disk image is malformed`, or `database is locked`.

**Solution:** Delete the index and rebuild:

```bash
claude-hindsight clean --db
claude-hindsight init
```

The `clean --db` command removes `~/.local/share/claude-hindsight/index.db`. Your original session files in `~/.claude/projects/` are untouched — everything can be rebuilt from them.

If the database is locked (another process has it open), close all other Hindsight processes first:

```bash
pkill -f claude-hindsight
claude-hindsight init
```

---

## Web dashboard shows a blank page

**Symptom:** You open `http://localhost:7227` and see an empty page or a React error screen.

**If you installed via Homebrew or pre-built binary:** This shouldn't happen. Try:

```bash
# Hard reload in your browser (Cmd+Shift+R / Ctrl+Shift+R)
# Or try a different browser
```

**If you built from source:** The web dashboard assets must be built before running `cargo build`. See [Contributing → Dev setup](/docs/contributing#dev-setup):

```bash
cd web && npm install && npm run build && cd ..
cargo build --release
```

Running `cargo run -- serve` without first building the web assets results in a blank page because there are no embedded files.

---

## OTLP daemon won't start

**Symptom:** `claude-hindsight daemon` exits immediately or `integrate --status` shows the daemon as not running.

**Check the port:** The daemon needs port `7228` to be free. See [Port already in use](#port-already-in-use).

**Check your config:**

```bash
claude-hindsight config show
```

Verify `otel.port` is set correctly and matches what `integrate --status` shows.

**Start manually to see errors:**

```bash
claude-hindsight daemon --port 7228
```

Any startup errors will print to stderr.

---

## Activity Timeline is empty

**Symptom:** The Activity page (`/activity`) shows no events even after running Claude Code sessions.

**Cause:** Hooks are not installed or the daemon is not running.

**Step 1:** Check hook status:

```bash
claude-hindsight integrate --status
```

If hooks are missing, install them:

```bash
claude-hindsight integrate --otel
```

**Step 2:** Make sure the daemon is running:

```bash
claude-hindsight daemon
# Or start the full server (which also starts the daemon):
claude-hindsight serve
```

**Step 3:** Start a new Claude Code session. Existing sessions that ran before hooks were installed will not appear in the Activity Timeline (only the SQLite-indexed data is available for them). Future sessions will stream in live.

---

## Hooks not triggering

**Symptom:** `integrate --status` shows hooks installed, but no Activity events appear.

**Check the settings file path:** Claude Code uses different settings files depending on how it's invoked. Hindsight defaults to writing to `~/.claude/settings.json`. If Claude Code is reading from a different location:

```bash
# Show where Claude Code looks for settings
claude --help | grep settings
```

**Force reinstall with explicit path:**

```bash
claude-hindsight integrate --otel --force
```

**Check hook execution:** Add `--verbose` to see whether the hook commands are actually running:

```bash
# In one terminal: watch hook output
claude-hindsight daemon

# In another terminal: start Claude Code and watch for OTLP events
```

If you see no OTLP events in the daemon output when running Claude Code, the hooks are not executing. This usually means the settings file path is wrong. See [Configuration → Data locations](/docs/configuration#data-locations) for the expected paths.

---

## Getting more help

If none of the above resolves your issue:

1. Run `claude-hindsight --version` and note the version
2. Collect the error output
3. Open an issue at [github.com/Codestz/claude-hindsight/issues](https://github.com/Codestz/claude-hindsight/issues) with your OS, version, and steps to reproduce
