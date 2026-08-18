//! Host boot: spawn the official `dsh web` host and wait until it is ready.
//!
//! The desktop shell never embeds or reimplements the harness — it boots the
//! published host and views it. The host command is overridable so the RSI
//! engine can pin a verified version.

use std::net::TcpStream;
use std::time::{Duration, Instant};

/// Address the `dsh web` host binds by default.
pub const HOST_ADDR: &str = "127.0.0.1:3080";
/// URL the webview navigates to once the host answers.
pub const HOST_URL: &str = "http://127.0.0.1:3080";

/// Spawn the host and block until its HTTP endpoint accepts connections.
pub fn start_and_wait() -> Result<String, String> {
    let (program, args) = resolve_host_command();

    spawn_host(&program, &args)?;
    wait_until_ready(HOST_ADDR, Duration::from_secs(120))?;

    Ok(HOST_URL.to_string())
}

/// Resolve the host boot command as `(program, args)`: explicit env override →
/// bundled launcher → system `dsh`. The program is kept whole so a path with
/// spaces (e.g. inside the `.app` bundle) is never split.
fn resolve_host_command() -> (String, Vec<String>) {
    if let Ok(command) = std::env::var("DSH_DESKTOP_HOST_CMD") {
        if !command.trim().is_empty() {
            let mut parts = command.split_whitespace();
            let program = parts.next().unwrap_or_default().to_string();
            return (program, parts.map(str::to_string).collect());
        }
    }
    if let Some(launcher) = bundled_launcher() {
        return (launcher, vec!["web".to_string()]);
    }
    ("dsh".to_string(), vec!["web".to_string()])
}

/// Best-effort locate the vendored launcher (written by `scripts/bundle-host.sh`),
/// covering the bundled `.app` layout and the `cargo run` dev layout.
fn bundled_launcher() -> Option<String> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    for candidate in [
        // Bundled .app: `../vendor/host` is mapped to `Resources/_up_/vendor/host`
        // (Tauri's `..` traversal marker), so the launcher sits under it.
        dir.join("../Resources/_up_/vendor/host/dsh-launcher"),
        // Dev (`cargo run` from src-tauri): exe is target/debug/, repo root is 3 up.
        dir.join("../../../vendor/host/dsh-launcher"),
    ] {
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

/// Spawn the host process, detached from this shell's lifetime expectations.
fn spawn_host(program: &str, args: &[String]) -> Result<(), String> {
    std::process::Command::new(program)
        .args(args)
        .spawn()
        .map_err(|err| format!("failed to spawn host `{program}`: {err}"))?;

    Ok(())
}

/// Poll a TCP connect until it succeeds or the deadline passes.
fn wait_until_ready(addr: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!(
        "host at {addr} did not become ready within {timeout:?}"
    ))
}
