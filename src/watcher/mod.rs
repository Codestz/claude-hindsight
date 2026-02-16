//! File watching utilities for real-time session monitoring

pub mod stream_renderer;

use crate::error::{HindsightError, Result};
use crate::parser::ExecutionNode;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Watches a session JSONL file for changes and streams new nodes
pub struct SessionWatcher {
    /// Path to the JSONL file being watched
    file_path: PathBuf,

    /// Current read position in the file (bytes)
    position: u64,

    /// File watcher
    _watcher: RecommendedWatcher,

    /// Receiver for file change events
    rx: Receiver<Result<Event>>,
}

impl SessionWatcher {
    /// Create a new session watcher for the given JSONL file
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file_path = file_path.as_ref().to_path_buf();

        if !file_path.exists() {
            return Err(HindsightError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Session file not found: {}", file_path.display()),
            )));
        }

        // Start reading from the beginning to show existing nodes
        let position = 0;

        // Set up file watcher
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res: notify::Result<Event>| {
                let _ = tx.send(res.map_err(|e| {
                    HindsightError::FileWatcher(format!("Watch error: {}", e))
                }));
            },
            notify::Config::default(),
        )
        .map_err(|e| HindsightError::FileWatcher(format!("Failed to create watcher: {}", e)))?;

        // Watch the file for changes
        watcher
            .watch(file_path.as_ref(), RecursiveMode::NonRecursive)
            .map_err(|e| HindsightError::FileWatcher(format!("Failed to watch file: {}", e)))?;

        Ok(SessionWatcher {
            file_path,
            position,
            _watcher: watcher,
            rx,
        })
    }

    /// Read new nodes appended to the file since last read
    ///
    /// Returns a vector of newly parsed nodes
    pub fn read_new_nodes(&mut self) -> Result<Vec<ExecutionNode>> {
        let mut file = File::open(&self.file_path)?;
        file.seek(SeekFrom::Start(self.position))?;

        let reader = BufReader::new(file);
        let mut new_nodes = Vec::new();

        for line in reader.lines() {
            let line = line?;

            // Update position
            self.position += line.len() as u64 + 1; // +1 for newline

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse the JSON line
            match serde_json::from_str::<ExecutionNode>(&line) {
                Ok(node) => new_nodes.push(node),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line: {}", e);
                }
            }
        }

        Ok(new_nodes)
    }

    /// Wait for file changes with a timeout
    ///
    /// Returns true if file was modified, false if timeout occurred
    pub fn wait_for_changes(&self, timeout: Duration) -> bool {
        match self.rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                // Check if it's a modify event
                matches!(event.kind, EventKind::Modify(_))
            }
            Ok(Err(_)) => false,
            Err(_) => false, // Timeout
        }
    }

    /// Check for new nodes immediately without blocking
    ///
    /// Returns newly appended nodes if any
    pub fn poll(&mut self) -> Result<Vec<ExecutionNode>> {
        // Try to receive events without blocking
        while let Ok(Ok(_event)) = self.rx.try_recv() {
            // Consume all pending events
        }

        // Read new content
        self.read_new_nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_watcher_creation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{\"type\":\"user\"}}").unwrap();
        temp_file.flush().unwrap();

        let watcher = SessionWatcher::new(temp_file.path());
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_read_new_nodes() {
        use crate::parser::models::ToolUse;

        let mut temp_file = NamedTempFile::new().unwrap();

        // Initial content
        writeln!(temp_file, "{{\"type\":\"user\"}}").unwrap();
        temp_file.flush().unwrap();

        let mut watcher = SessionWatcher::new(temp_file.path()).unwrap();

        // Position should be at end of file
        assert!(watcher.position > 0);

        // Append new content
        let new_node = ExecutionNode {
            uuid: Some("test-uuid".to_string()),
            parent_uuid: None,
            timestamp: Some(1000),
            node_type: "tool_use".to_string(),
            message: None,
            tool_use: Some(ToolUse {
                name: "Read".to_string(),
                input: serde_json::json!({"file": "test.rs"}),
                id: Some("tool-1".to_string()),
            }),
            tool_result: None,
            tool_use_result: None,
            thinking: None,
            progress: None,
            token_usage: None,
            extra: None,
        };

        writeln!(temp_file, "{}", serde_json::to_string(&new_node).unwrap()).unwrap();
        temp_file.flush().unwrap();

        // Read new nodes
        let new_nodes = watcher.read_new_nodes().unwrap();
        assert_eq!(new_nodes.len(), 1);
        assert_eq!(new_nodes[0].node_type, "tool_use");
    }
}
