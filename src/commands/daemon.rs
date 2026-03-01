//! Implementation of the `daemon` command
//!
//! Starts a minimal background HTTP listener (default port 7228) that
//! accepts OTLP http/json payloads and stores them in the Hindsight database.
//! When auto-spawned by the SessionStart hook, uses an idle timeout so the
//! daemon exits cleanly after a period of inactivity.

use crate::error::Result;

pub fn run(port: u16, idle_timeout: u64) -> Result<()> {
    println!("Starting Hindsight telemetry daemon on port {}...", port);
    println!(
        "Set the following environment variables in Claude Code:\n\
         \n  CLAUDE_CODE_ENABLE_TELEMETRY=1\
         \n  OTEL_METRICS_EXPORTER=otlp\
         \n  OTEL_EXPORTER_OTLP_PROTOCOL=http/json\
         \n  OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:{}\
         \n",
        port
    );
    if idle_timeout > 0 {
        println!("Idle timeout: {}s\n", idle_timeout);
    } else {
        println!("Press Ctrl+C to stop.\n");
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        crate::error::HindsightError::Config(format!("Failed to create async runtime: {}", e))
    })?;

    rt.block_on(async move {
        let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
        crate::server::daemon::serve_with_idle_timeout(addr, idle_timeout)
            .await
            .map_err(|e| {
                crate::error::HindsightError::Config(format!("Daemon error: {}", e))
            })
    })
}
