//! `hindsight serve` — start the web dashboard server

use crate::error::Result;

pub fn run(port: u16, open: bool) -> Result<()> {
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
        .block_on(async { crate::server::serve(addr).await })
        .map_err(|e| crate::error::HindsightError::Config(e.to_string()))?;

    Ok(())
}
