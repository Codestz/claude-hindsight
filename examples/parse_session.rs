//! Example: Parse a Claude Code session JSONL file

use hindsight::parser::parse_session;
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <session.jsonl>", args[0]);
        process::exit(1);
    }
    
    let path = Path::new(&args[1]);
    
    match parse_session(path) {
        Ok(session) => {
            println!("✅ Successfully parsed session: {}", session.session_id);
            println!("   Nodes: {}", session.nodes.len());
            println!("   Tools: {}", session.total_tools);
            println!("   Tokens: {}", session.total_tokens);
            println!("   Cost: ${:.2}", session.estimated_cost);
            println!("   Errors: {}", session.error_count);
            
            if let (Some(start), Some(end)) = (session.start_time, session.end_time) {
                let duration_sec = (end - start) / 1000;
                println!("   Duration: {}s", duration_sec);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to parse session: {}", e);
            process::exit(1);
        }
    }
}
