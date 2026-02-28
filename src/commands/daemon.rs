//! Implementation of the `daemon` command
//!
//! Starts a minimal background HTTP listener (default port 7228) that
//! accepts OTLP http/json payloads and stores them in the Hindsight database.
//! Users who want persistent, real-time telemetry run this once (e.g. via
//! launchd or systemd).

use crate::error::Result;

pub fn run(port: u16) -> Result<()> {
    eprintln!(
        "Note: `hindsight daemon` is deprecated. \
         `hindsight serve` now includes a built-in OTLP receiver on port 7228. \
         Use `--otel-port 0` to disable it, or `--otel-port <N>` to change the port."
    );
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
    println!("Press Ctrl+C to stop.\n");

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        crate::error::HindsightError::Config(format!("Failed to create async runtime: {}", e))
    })?;

    rt.block_on(async move {
        let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
        crate::server::daemon::serve(addr).await.map_err(|e| {
            crate::error::HindsightError::Config(format!("Daemon error: {}", e))
        })
    })
}
