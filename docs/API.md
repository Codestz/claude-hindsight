# API Module

The `api` module provides JSON-serializable data structures for web dashboard integration.

## Response Types

- `NodeResponse` - Serializable tree node with label, color, and metadata
- `TreeResponse` - Complete tree with roots and statistics
- `SessionStatsResponse` - Session metadata and aggregate statistics

## Usage

```rust
use hindsight::api::responses::{NodeResponse, TreeResponse};
use hindsight::analyzer::build_simple_tree;
use hindsight::parser::parse_session;

// Parse session
let session = parse_session("/path/to/session.jsonl")?;

// Build tree
let tree = build_simple_tree(session.nodes);

// Convert to API response
let response: Vec<NodeResponse> = tree
    .iter()
    .map(NodeResponse::from_tree_node)
    .collect();

// Serialize to JSON
let json = serde_json::to_string_pretty(&response)?;
println!("{}", json);
```

## Example Response

```json
{
  "uuid": "abc123",
  "node_type": "tool_use",
  "label": " Read",
  "color": "cyan",
  "summary": "Read src/main.rs",
  "depth": 1,
  "has_error": false,
  "timestamp": 1234567890000,
  "children": [],
  "data": {
    "uuid": "abc123",
    "node_type": "tool_use",
    "tool_use": {
      "name": "Read",
      "input": {"file_path": "src/main.rs"}
    }
  }
}
```

## Presentation Configuration

The `formatting` module provides presentation configuration for different output formats:

```rust
use hindsight::api::formatting::{PresentationConfig, ColorScheme, IconStyle};

let config = PresentationConfig {
    color_scheme: ColorScheme::Dark,
    icon_style: IconStyle::NerdFont,
};
```

### Icon Styles

- `NerdFont` - Terminal Nerd Fonts (default for TUI)
- `Unicode` - Unicode emoji (for web)
- `Ascii` - ASCII fallback (for plain text)
- `None` - No icons

### Color Schemes

- `Light` - Light mode colors
- `Dark` - Dark mode colors (default)

## Integration with Web Dashboard

The API module is designed to be used with web frameworks like Axum or Actix-web:

```rust
use axum::{Router, Json};
use hindsight::api::responses::NodeResponse;

async fn get_session_tree(session_id: String) -> Json<Vec<NodeResponse>> {
    // Load and parse session
    let session = load_session(&session_id).await?;
    let tree = build_simple_tree(session.nodes);

    // Convert to API response
    let response = tree
        .iter()
        .map(NodeResponse::from_tree_node)
        .collect();

    Json(response)
}
```

## Design Principles

1. **Separation of Concerns** - Presentation logic is separate from data structures
2. **Serialization-First** - All types derive `Serialize` and `Deserialize`
3. **Frontend-Friendly** - Flat structures, semantic colors, pre-computed labels
4. **No TUI Dependencies** - API module has no ratatui or terminal dependencies

## Future Enhancements

- [ ] Pagination support for large trees
- [ ] Filtering and search at API level
- [ ] GraphQL schema generation
- [ ] WebSocket streaming for live sessions
