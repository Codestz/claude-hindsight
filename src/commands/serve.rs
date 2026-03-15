//! `hindsight serve` — start the web dashboard server (+ optional embedded OTLP listener)

use crate::error::Result;

/// Check whether something is already listening on `port`.
fn port_in_use(port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;

    TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(200),
    )
    .is_ok()
}

pub fn run(port: u16, open: bool, otel_port: u16) -> Result<()> {
    // If an OTLP daemon is already running on the target port, let it be.
    // Both the daemon and `serve` write to the same SQLite database, so
    // there is no need to kill the daemon — it will continue collecting
    // OTLP data while `serve` provides the web dashboard.  The `serve`
    // server already has /v1/metrics and /v1/logs on its own port (the
    // dashboard port), so OTLP is still reachable on both addresses.
    //
    // Only start the embedded OTLP listener if nothing is on the port.
    let effective_otel_port = if otel_port > 0 && port_in_use(otel_port) {
        eprintln!(
            "OTLP daemon already running on port {otel_port} — reusing it (no restart needed)."
        );
        0 // skip starting the OTLP listener inside serve
    } else {
        otel_port
    };

    let addr: std::net::SocketAddr =
        format!("0.0.0.0:{port}")
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                crate::error::HindsightError::Config(e.to_string())
            })?;

    if open {
        // Best-effort browser open — ignore errors
        let url = format!("http://localhost:{port}");
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn();
    }

    // Use a multi-thread tokio runtime; block_on keeps main() synchronous.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| crate::error::HindsightError::Config(e.to_string()))?
        .block_on(async { crate::server::serve(addr, effective_otel_port).await })
        .map_err(|e| crate::error::HindsightError::Config(e.to_string()))?;

    Ok(())
}
