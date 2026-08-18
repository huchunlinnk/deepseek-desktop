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
    let command = resolve_host_command();

    spawn_host(&command)?;
    wait_until_ready(HOST_ADDR, Duration::from_secs(120))?;

    Ok(HOST_URL.to_string())
}

/// Resolve the host boot command: explicit env override → bundled launcher → system `dsh`.
fn resolve_host_command() -> String {
    if let Ok(command) = std::env::var("DSH_DESKTOP_HOST_CMD") {
        if !command.trim().is_empty() {
            return command;
        }
    }
    if let Some(launcher) = bundled_launcher() {
        return format!("{launcher} web");
    }
    "dsh web".to_string()
}

/// Best-effort locate the vendored launcher (written by `scripts/bundle-host.sh`)
/// beside the executable, covering the macOS `Resources/` and Windows layouts.
fn bundled_launcher() -> Option<String> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    for candidate in [
        dir.join("host/dsh-launcher"),
        dir.join("../Resources/host/dsh-launcher"),
    ] {
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

/// Spawn the host process, detached from this shell's lifetime expectations.
fn spawn_host(command: &str) -> Result<(), String> {
    let mut parts = command.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| "DSH_DESKTOP_HOST_CMD is empty".to_string())?;

    std::process::Command::new(program)
        .args(parts)
        .spawn()
        .map_err(|err| format!("failed to spawn host `{command}`: {err}"))?;

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
