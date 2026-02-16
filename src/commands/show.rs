//! Implementation of the `show` command
//!
//! Displays execution tree for a Claude Code session.

use crate::analyzer::{render_stats, render_tree, ExecutionTree, RenderConfig};
use crate::error::{HindsightError, Result};
use crate::parser::parse_session;
use crate::storage::SessionIndex;

pub fn run(session_id: String, dashboard: bool, _port: u16) -> Result<()> {
    if dashboard {
        println!("🌐 Web dashboard not yet implemented");
        println!("💡 Use without --dashboard flag to view in terminal\n");
        return Ok(());
    }

    // Find session
    let index = SessionIndex::new()?;
    let session_file = index
        .find_by_id(&session_id)?
        .ok_or_else(|| HindsightError::SessionNotFound(session_id.clone()))?;

    println!("📖 Loading session: {}\n", session_file.session_id);

    // Parse session
    let session = parse_session(&session_file.path)?;

    // Build execution tree
    let tree = ExecutionTree::from_nodes(session.nodes);

    // Display statistics
    print!("{}", render_stats(&tree));

    // Display tree structure
    println!("\n🌳 Execution Tree:\n");

    let config = RenderConfig {
        max_nodes: Some(100),
        max_depth: Some(10),
        show_timestamps: true,
        show_types: false,
        compact: false,
    };

    print!("{}", render_tree(&tree, &config));

    // Show tool calls if any
    if tree.stats.tool_calls > 0 {
        println!("\n🔧 Tool Calls:\n");
        for (i, node) in tree.get_tool_calls().iter().enumerate().take(20) {
            if let Some(ref tool_use) = node.node.tool_use {
                let depth_indicator = "  ".repeat(node.depth);
                println!("   {}{}. {}", depth_indicator, i + 1, tool_use.name);
            }
        }

        if tree.stats.tool_calls > 20 {
            println!("\n   ... ({} more tool calls)", tree.stats.tool_calls - 20);
        }
    }

    // Show errors if any
    if tree.stats.errors > 0 {
        println!("\n❌ Errors:\n");
        for (i, node) in tree.get_errors().iter().enumerate() {
            if let Some(ref result) = node.node.tool_result {
                println!("   {}. Error at depth {}", i + 1, node.depth);
                if let Some(ref error) = result.error {
                    let error_preview = if error.len() > 100 {
                        format!("{}...", &error[..100])
                    } else {
                        error.clone()
                    };
                    println!("      {}", error_preview);
                }
            }
        }
    }

    println!("\n💡 Tip: Use --dashboard flag to view in web browser");

    Ok(())
}
