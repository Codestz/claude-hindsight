//! Server-Sent Events route for live session watching
//!
//! GET /api/events?session_id=<id>
//!
//! Streams NodeResponse events as new lines are appended to the JSONL file.

use axum::{
    body::Bytes,
    extract::{Query, State},
    response::Response,
    http::header,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use futures_util::StreamExt;
use tokio_stream::wrappers::IntervalStream;
use crate::server::{AppState, error::ApiError};
use crate::api::responses::NodeResponse;
use crate::storage::SessionIndex;

#[derive(Deserialize)]
pub struct EventsQuery {
    pub session_id: String,
}

pub async fn live_events(
    State(_state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> Result<Response, ApiError> {
    let session_id = q.session_id.clone();
    let session_path = tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
        let index = SessionIndex::new()?;
        let session = index
            .find_by_id(&session_id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(session_id.clone()))?;
        Ok(session.path)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    // Shared mutable byte offset — updated each poll
    let offset = Arc::new(Mutex::new(0u64));

    let interval = tokio::time::interval(Duration::from_millis(500));
    let interval_stream = IntervalStream::new(interval);

    let sse_stream = interval_stream.then(move |_| {
        let path = session_path.clone();
        let offset = Arc::clone(&offset);
        async move {
            let current_offset = *offset.lock().unwrap();
            let (nodes, new_offset) = tokio::task::spawn_blocking(move || {
                tail_new_nodes(&path, current_offset)
            })
            .await
            .unwrap_or_else(|_| (vec![], current_offset));

            *offset.lock().unwrap() = new_offset;

            let events: Vec<String> = nodes
                .into_iter()
                .map(|n| {
                    let data = serde_json::to_string(&n).unwrap_or_default();
                    format!("data: {}\n\n", data)
                })
                .collect();

            let payload = if events.is_empty() {
                ": heartbeat\n\n".to_string()
            } else {
                events.join("")
            };

            Ok::<_, Infallible>(Bytes::from(payload))
        }
    });

    let body = axum::body::Body::from_stream(sse_stream);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header("X-Accel-Buffering", "no")
        .body(body)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(response)
}

/// Read new JSONL lines starting at `offset` bytes.
/// Must run inside spawn_blocking — uses Rc internally via TreeNode.
fn tail_new_nodes(path: &std::path::Path, offset: u64) -> (Vec<NodeResponse>, u64) {
    use std::io::{Read, Seek, SeekFrom};
    use std::rc::Rc;
    use crate::analyzer::TreeNode;

    let Ok(mut file) = std::fs::File::open(path) else {
        return (vec![], offset);
    };
    let Ok(file_len) = file.seek(SeekFrom::End(0)) else {
        return (vec![], offset);
    };
    if file_len <= offset {
        return (vec![], offset);
    }
    let Ok(_) = file.seek(SeekFrom::Start(offset)) else {
        return (vec![], offset);
    };

    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return (vec![], offset);
    }

    let new_offset = offset + buf.len() as u64;
    let mut nodes = Vec::new();

    for line in buf.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Ok(exec_node) = serde_json::from_str::<crate::parser::ExecutionNode>(line) {
            let tree_node = TreeNode {
                node: Rc::new(exec_node),
                children: vec![],
                depth: 0,
            };
            nodes.push(NodeResponse::from_tree_node(&tree_node));
        }
    }

    (nodes, new_offset)
}
