use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=HINDSIGHT_SKIP_WEB_BUILD");

    // Watch web source directories if present (skipped in crates.io installs)
    for dir in &["web/src", "web/public", "web/styles", "web/components"] {
        if Path::new(dir).exists() {
            println!("cargo:rerun-if-changed={dir}");
        }
    }
    if Path::new("web/package.json").exists() {
        println!("cargo:rerun-if-changed=web/package.json");
    }
    if Path::new("web/next.config.ts").exists() {
        println!("cargo:rerun-if-changed=web/next.config.ts");
    }

    // If web/out/index.html already exists, nothing to do.
    if Path::new("web/out/index.html").exists() {
        return;
    }

    // Honour opt-out env var (useful in CI that builds without Node).
    if std::env::var("HINDSIGHT_SKIP_WEB_BUILD").is_ok() {
        create_placeholder();
        return;
    }

    // Try to run npm build automatically.
    if node_available() {
        let result = Command::new("npm")
            .args(["install"])
            .current_dir("web")
            .status()
            .and_then(|_| {
                Command::new("npm")
                    .args(["run", "build"])
                    .current_dir("web")
                    .status()
            });

        match result {
            Ok(s) if s.success() => return,
            _ => {
                println!(
                    "cargo:warning=npm build failed — embedding placeholder web UI. \
                     Run `cd web && npm run build` manually, then `cargo build --release`."
                );
            }
        }
    } else {
        println!(
            "cargo:warning=Node.js not found — embedding placeholder web UI. \
             Install Node.js 20+ and rerun `cargo build --release` for the full dashboard."
        );
    }

    create_placeholder();
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn create_placeholder() {
    std::fs::create_dir_all("web/out").expect("could not create web/out/");

    std::fs::write(
        "web/out/index.html",
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Hindsight — API Only Mode</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 600px; margin: 80px auto;
           padding: 0 20px; color: #1a1a1a; line-height: 1.6; }
    h1   { font-size: 1.5rem; }
    code { background: #f0f0f0; padding: 2px 6px; border-radius: 3px; font-size: .9em; }
    pre  { background: #f0f0f0; padding: 16px; border-radius: 6px; overflow-x: auto; }
  </style>
</head>
<body>
  <h1>Hindsight — API Only Mode</h1>
  <p>The web dashboard was not included in this build because Node.js was unavailable
     or the frontend build failed.</p>
  <p>The REST API is fully functional at <code>/api/*</code>.</p>
  <h2>Build with the full dashboard</h2>
  <pre>cd web &amp;&amp; npm install &amp;&amp; npm run build
cd .. &amp;&amp; cargo build --release</pre>
  <p>Or simply install Node.js 20+ and run <code>cargo build --release</code> — the
     build script handles the frontend automatically.</p>
</body>
</html>
"#,
    )
    .expect("could not write web/out/index.html");

    std::fs::write(
        "web/out/404.html",
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\">\
         <title>404 — Not Found</title></head>\
         <body><h1>404 — Not Found</h1></body></html>",
    )
    .expect("could not write web/out/404.html");
}
