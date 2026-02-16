//! Example: Analyze execution tree structure
//!
//! Demonstrates building and analyzing the hierarchical execution tree.
//!
//! Usage:
//!   cargo run --example analyze_tree ~/.claudep/projects/.../session.jsonl

use hindsight::{parse_session, ExecutionTree, RenderConfig, render_tree, render_stats};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <session.jsonl>", args[0]);
        std::process::exit(1);
    }

    let session_path = Path::new(&args[1]);
    println!("📖 Parsing session: {}", session_path.display());

    // Parse the session
    let session = parse_session(session_path)?;
    println!("   Loaded {} nodes\n", session.nodes.len());

    // Build execution tree
    println!("🌲 Building execution tree...");
    let tree = ExecutionTree::from_nodes(session.nodes);

    // Display tree statistics using renderer
    print!("\n{}", render_stats(&tree));

    // Display tree structure with custom config
    println!("\n🌳 Tree Structure:\n");
    let config = RenderConfig {
        max_nodes: Some(50),
        max_depth: Some(5),
        show_timestamps: true,
        show_types: false,
        compact: false,
    };
    print!("{}", render_tree(&tree, &config));

    // Display tool calls
    if tree.stats.tool_calls > 0 {
        println!("\n🔧 Tool Calls:");
        for (i, node) in tree.get_tool_calls().iter().enumerate().take(10) {
            if let Some(ref tool_use) = node.node.tool_use {
                println!("   {}. {} (depth: {})", i + 1, tool_use.name, node.depth);
            }

            if i == 9 && tree.stats.tool_calls > 10 {
                println!("   ... ({} more tool calls)", tree.stats.tool_calls - 10);
            }
        }
    }

    // Display errors if any
    if tree.stats.errors > 0 {
        println!("\n❌ Errors Found:");
        for (i, node) in tree.get_errors().iter().enumerate() {
            if let Some(ref result) = node.node.tool_result {
                println!("   {}. Error at depth {}", i + 1, node.depth);
                if let Some(ref error) = result.error {
                    println!("      {}", error);
                }
            }
        }
    }

    Ok(())
}
