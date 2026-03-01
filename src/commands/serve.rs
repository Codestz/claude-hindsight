//! `hindsight serve` — start the web dashboard server (+ optional embedded OTLP listener)

use crate::error::Result;

/// Kill an existing background daemon bound to `port` so `serve` can take over.
fn kill_existing_daemon(port: u16) {
    use std::net::TcpStream;
    use std::time::Duration;

    // Quick check — if nothing is listening, nothing to kill.
    if TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(200),
    )
    .is_err()
    {
        return;
    }

    // Use lsof to find the PID bound to this port, then kill it.
    let output = std::process::Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}")])
        .output();

    if let Ok(out) = output {
        let pids = String::from_utf8_lossy(&out.stdout);
        let my_pid = std::process::id();
        for pid_str in pids.split_whitespace() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                if pid != my_pid {
                    eprintln!("Stopping existing daemon (PID {pid}) on port {port}...");
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .output();
                }
            }
        }
        // Give the old process a moment to release the port.
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn run(port: u16, open: bool, otel_port: u16) -> Result<()> {
    // If an OTLP daemon is already running on the target port, kill it so
    // `serve` can take over both the dashboard and OTLP receiver.  When the
    // user later stops `serve`, the next hook invocation will re-spawn a
    // lightweight background daemon automatically.
    if otel_port > 0 {
        kill_existing_daemon(otel_port);
    }

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
        .block_on(async { crate::server::serve(addr, otel_port).await })
        .map_err(|e| crate::error::HindsightError::Config(e.to_string()))?;

    Ok(())
}
