# Claude Code JSONL Structure Documentation

This document provides a comprehensive reference for the JSONL transcript format produced by Claude Code sessions.

## Overview

Claude Code generates JSONL (JSON Lines) transcript files that record every interaction, tool call, and internal operation during a session. Each line in the file is a valid JSON object representing a single event node.

**Typical Session Statistics:**
- User nodes: ~718 per session
- Assistant nodes: ~1,439 per session
- Progress nodes: ~571 per session
- File snapshots: ~178 per session
- System nodes: ~40 per session
- Queue operations: ~12 per session

---

## Node Types

### 1. User Messages (`type: "user"`)

Represents user input to Claude Code.

**Key Fields:**
- `uuid`: Unique identifier
- `parent_uuid`: Parent node UUID
- `timestamp`: Event timestamp (ISO 8601 string or milliseconds)
- `type`: `"user"`
- `message`: Message object with content
- `toolUseResult`: Optional structured tool result data

**Message Content Structure:**

Content is ALWAYS an array, even for single items:

```json
{
  "type": "user",
  "uuid": "abc-123",
  "timestamp": 1234567890000,
  "message": {
    "role": "user",
    "content": [
      {
        "type": "text",
        "text": "Read the config file"
      }
    ]
  }
}
```

**Tool Use Result Field:**

User nodes can include `toolUseResult` to show file operations:

```json
{
  "type": "user",
  "uuid": "def-456",
  "toolUseResult": {
    "type": "create",
    "filePath": "/path/to/file.rs",
    "content": "...",
    "structuredPatch": [...]
  }
}
```

**Content Item Types in User Messages:**
- `type: "text"` - Text content (field: `text`)
- `type: "tool_result"` - Tool execution results (fields: `tool_use_id`, `content`, `is_error`)

---

### 2. Assistant Messages (`type: "assistant"`)

Represents Claude's responses.

**Key Fields:**
- `uuid`: Unique identifier
- `parent_uuid`: Parent node UUID
- `timestamp`: Event timestamp
- `type`: `"assistant"`
- `message`: Message object with content array
- `token_usage`: Token statistics

**Message Content Structure:**

Assistant messages have complex content arrays with multiple types:

```json
{
  "type": "assistant",
  "uuid": "ghi-789",
  "timestamp": 1234567890000,
  "message": {
    "role": "assistant",
    "content": [
      {
        "type": "thinking",
        "thinking": "Let me analyze this...",
        "signature": "..."
      },
      {
        "type": "text",
        "text": "I'll read the file now."
      },
      {
        "type": "tool_use",
        "id": "tool-123",
        "name": "Read",
        "input": {
          "file_path": "/path/to/file"
        }
      }
    ]
  },
  "token_usage": {
    "input_tokens": 1500,
    "output_tokens": 250,
    "cache_creation_input_tokens": 0,
    "cache_read_input_tokens": 1200
  }
}
```

**Content Item Types in Assistant Messages:**
- `type: "text"` - Text response (field: `text`)
- `type: "thinking"` - Thinking blocks (field: `thinking`, has `signature`)
- `type: "tool_use"` - Tool calls (fields: `id`, `name`, `input`)

**Token Usage:**
- `input_tokens`: Tokens in the prompt
- `output_tokens`: Tokens generated
- `cache_creation_input_tokens`: Tokens written to cache
- `cache_read_input_tokens`: Tokens read from cache

---

### 3. Progress Nodes (`type: "progress"`)

Progress updates for long-running operations.

**Important:** Progress nodes have a NESTED type structure. The top-level `type` is always `"progress"`, but the actual progress type is in `data.type`.

**Structure:**

```json
{
  "type": "progress",
  "uuid": "jkl-012",
  "timestamp": 1234567890000,
  "data": {
    "type": "bash_progress",
    "elapsedTimeSeconds": 5.2,
    "fullOutput": "Command output...",
    "exitCode": null
  }
}
```

**Progress Subtypes** (via `data.type`):

1. **`bash_progress`** - Bash command execution progress
   ```json
   {
     "data": {
       "type": "bash_progress",
       "elapsedTimeSeconds": 2.5,
       "fullOutput": "$ cargo build\n   Compiling...",
       "exitCode": null
     }
   }
   ```

2. **`hook_progress`** - Hook operation progress
   ```json
   {
     "data": {
       "type": "hook_progress",
       "hookName": "pre-commit",
       "status": "running"
     }
   }
   ```

3. **`waiting_for_task`** - Task waiting status
   ```json
   {
     "data": {
       "type": "waiting_for_task",
       "taskDescription": "Running tests",
       "taskId": "task-456"
     }
   }
   ```

4. **`agent_progress`** - Subagent execution progress
   ```json
   {
     "data": {
       "type": "agent_progress",
       "agentType": "Explore",
       "status": "running",
       "message": "Searching codebase for authentication patterns..."
     }
   }
   ```

---

### 4. File History Snapshots (`type: "file-history-snapshot"`)

Periodic backups of tracked files in the session.

**Structure:**

```json
{
  "type": "file-history-snapshot",
  "uuid": "mno-345",
  "timestamp": 1234567890000,
  "snapshot": {
    "trackedFileBackups": {
      "src/main.rs": "fn main() {\n    println!(\"Hello\");\n}",
      "Cargo.toml": "[package]\nname = \"myapp\"\n...",
      "README.md": "# My Project\n..."
    },
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

**Key Fields:**
- `snapshot.trackedFileBackups`: Object mapping file paths to their content
- `snapshot.timestamp`: When the snapshot was taken

**Purpose:**
- Allows reconstruction of file state at any point in the session
- Enables "time travel" debugging
- Typically created every ~15-20 operations

---

### 5. System Nodes (`type: "system"`)

System metadata and internal events.

**Structure:**

```json
{
  "type": "system",
  "uuid": "pqr-678",
  "timestamp": 1234567890000,
  "subtype": "turn_duration",
  "durationMs": 221373
}
```

**Common Subtypes:**

1. **`turn_duration`** - Marks the end of a conversation turn
   ```json
   {
     "type": "system",
     "subtype": "turn_duration",
     "durationMs": 5432
   }
   ```

2. **Other system events** - Various internal markers
   - Session initialization
   - Permission changes
   - Configuration updates

**Note:** Most system nodes have minimal displayable content and serve as metadata markers.

---

### 6. Queue Operations (`type: "queue-operation"`)

Queued user messages waiting to be processed.

**Structure:**

```json
{
  "type": "queue-operation",
  "uuid": "stu-901",
  "timestamp": 1234567890000,
  "operation": "enqueue",
  "message": {
    "role": "user",
    "content": [{"type": "text", "text": "..."}]
  }
}
```

**Fields:**
- `operation`: Type of queue operation (`"enqueue"`, `"dequeue"`)
- `message`: The queued message content

---

## Content Array Format

**Critical Understanding:** Both user and assistant messages have `message.content` as an **array of content items**, even when there's only one item.

### Content Item Types

Each item in the `content` array has a `type` field:

| Type | Fields | Description |
|------|--------|-------------|
| `text` | `text` | Plain text content |
| `thinking` | `thinking`, `signature` | Claude's thinking process |
| `tool_use` | `id`, `name`, `input` | Tool call request |
| `tool_result` | `tool_use_id`, `content`, `is_error` | Tool execution result |

### Example: Complex Assistant Message

```json
{
  "message": {
    "role": "assistant",
    "content": [
      {
        "type": "thinking",
        "thinking": "I need to read the file first to understand the structure.",
        "signature": "sig-abc"
      },
      {
        "type": "text",
        "text": "I'll read the configuration file to check the settings."
      },
      {
        "type": "tool_use",
        "id": "toolu_01ABC",
        "name": "Read",
        "input": {
          "file_path": "/config/app.json"
        }
      }
    ]
  }
}
```

### Example: User Message with Tool Result

```json
{
  "message": {
    "role": "user",
    "content": [
      {
        "type": "tool_result",
        "tool_use_id": "toolu_01ABC",
        "is_error": false,
        "content": "{\"version\": \"1.0\", \"debug\": true}"
      }
    ]
  }
}
```

---

## Hierarchical Structure

Nodes are linked via UUID references:

```
parent_uuid → uuid
     |
     └─> Creates parent-child relationships
```

**Example Hierarchy:**

```
user (uuid: A)
  └─> assistant (uuid: B, parent_uuid: A)
       ├─> tool_use (uuid: C, parent_uuid: B)
       └─> tool_result (uuid: D, parent_uuid: C)
```

**Root Nodes:** Nodes with `parent_uuid: null` or no parent are top-level roots.

---

## Common Patterns

### 1. Conversation Turn

```
user message
  └─> assistant message
       └─> thinking
       └─> text
       └─> tool_use
            └─> progress (bash_progress)
            └─> tool_result
                 └─> assistant response
                      └─> text
```

### 2. File Operation

```
assistant (tool_use: Write)
  └─> tool_result (success)
       └─> file-history-snapshot (backup created)
```

### 3. Error Recovery

```
tool_use
  └─> tool_result (is_error: true)
       └─> assistant (thinking about fix)
            └─> tool_use (retry with fix)
```

---

## Parsing Guidelines

### Best Practices

1. **Always treat `message.content` as an array** - Even single items are wrapped in arrays
2. **Check `data.type` for progress nodes** - The top-level `type` is just `"progress"`
3. **Handle missing fields gracefully** - Not all fields are present in all nodes
4. **Use `extra` field for unknown data** - Capture additional fields for future compatibility

### Content Extraction

To extract text from a message:

```rust
fn extract_text(content: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = content.as_array() {
        arr.iter()
            .filter_map(|item| {
                match item.get("type")?.as_str()? {
                    "text" => item.get("text")?.as_str().map(String::from),
                    "thinking" => item.get("thinking")?.as_str().map(String::from),
                    _ => None
                }
            })
            .collect()
    } else {
        vec![]
    }
}
```

### Tool Detection

To find all tool calls:

```rust
fn find_tool_uses(content: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = content.as_array() {
        arr.iter()
            .filter_map(|item| {
                if item.get("type")?.as_str()? == "tool_use" {
                    item.get("name")?.as_str().map(String::from)
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}
```

---

## Schema Summary

### Top-Level Node Fields

All nodes share these common fields:

```typescript
{
  uuid?: string;           // Unique identifier
  parent_uuid?: string;    // Parent node reference
  timestamp?: number | string;  // Event time
  type: string;            // Node type
  message?: Message;       // For user/assistant
  tool_use?: ToolUse;      // For tool calls
  tool_result?: ToolResult;  // For tool results
  thinking?: string;       // For thinking blocks
  progress?: Progress;     // For progress updates
  token_usage?: TokenUsage;  // Token statistics
  [key: string]: any;      // Additional fields
}
```

### Message Structure

```typescript
{
  role: "user" | "assistant" | "system";
  content: Array<ContentItem>;
}
```

### Content Item Union

```typescript
type ContentItem =
  | { type: "text"; text: string; }
  | { type: "thinking"; thinking: string; signature: string; }
  | { type: "tool_use"; id: string; name: string; input: object; }
  | { type: "tool_result"; tool_use_id: string; content: string; is_error: boolean; };
```

---

## Version History

- **v1.0** (Initial) - Basic structure with user/assistant/tool nodes
- **v2.0** (Current) - Added progress types, file snapshots, enhanced content arrays

---

## References

- [Claude Code Documentation](https://github.com/anthropics/claude-code)
- [JSONL Format Specification](http://jsonlines.org/)

---

## Contributing

When updating this documentation:
1. Include real examples from actual sessions
2. Sanitize any sensitive data (API keys, personal info)
3. Verify field names match actual JSONL output
4. Update schema summary when adding new fields
5. Add examples for new node types
