//! Host boot: spawn the official `dsh web` host and wait until it is ready.
//!
//! The desktop shell never embeds or reimplements the harness — it boots the
//! published host and views it. The host command is overridable so the RSI
//! engine can pin a verified version.

use std::io::Read;
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Address the `dsh web` host binds by default.
pub const HOST_ADDR: &str = "127.0.0.1:3080";
/// URL the webview navigates to once the host answers.
pub const HOST_URL: &str = "http://127.0.0.1:3080";

/// The spawned host child, kept so the shell can kill it on exit — otherwise it
/// orphans and holds the port, blocking the next launch.
static HOST_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

/// Kill the host child process, if any. Called once on app exit.
pub fn kill_host() {
    if let Ok(mut guard) = HOST_CHILD.get_or_init(|| Mutex::new(None)).lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Spawn the host and block until its HTTP endpoint accepts connections.
/// If the preferred command fails at boot, fall back to the system `dsh`
/// before giving up — a stale vendored host (version skew against
/// `~/.dsh/.credentials.yaml` or the profile dir) must not brick the app.
pub fn start_and_wait() -> Result<String, String> {
    let (program, args) = resolve_host_command();

    match spawn_and_wait(&program, &args) {
        Ok(()) => return Ok(HOST_URL.to_string()),
        Err(primary_err) => {
            // Self-repair fallback: the system dsh tracks the newest state
            // (it is what upgraded the credentials file in the first place),
            // so it is the best candidate for reviving a broken boot.
            let fallback = "dsh";
            eprintln!(
                "[deepseek-desktop] host `{program}` failed, falling back to `{fallback}`: {primary_err}"
            );
            if program != fallback {
                if let Ok(()) = spawn_and_wait(fallback, &host_args()) {
                    return Ok(HOST_URL.to_string());
                }
            }
            Err(primary_err)
        }
    }
}

/// Spawn one command and wait for readiness, as `start_and_wait` does per try.
fn spawn_and_wait(program: &str, args: &[String]) -> Result<(), String> {
    spawn_host(program, args)?;
    wait_until_ready(HOST_ADDR, Duration::from_secs(120))
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
        return (launcher, host_args());
    }
    ("dsh".to_string(), host_args())
}

/// Arguments for `dsh web`: never let the host open the default browser —
/// the webview is the UI. Newer hosts open a browser by default, which
/// hijacked the user's browser on every app launch.
fn host_args() -> Vec<String> {
    vec!["web".to_string(), "--no-open".to_string()]
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

/// Spawn the host process and record its handle so `kill_host` can reap it.
/// Stderr is piped and drained on a reader thread: a host that dies at boot
/// (bad credentials, corrupted profile, version skew) must surface its error
/// instead of being muted while the readiness poll runs out its two minutes.
fn spawn_host(program: &str, args: &[String]) -> Result<(), String> {
    let mut child = Command::new(program)
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|err| format!("failed to spawn host `{program}`: {err}"))?;

    if let Some(stderr) = child.stderr.take() {
        // Shared ring of the last stderr bytes, read until the child exits.
        let tail: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        ERR_TAIL.get_or_init(|| tail.clone());
        std::thread::spawn(move || {
            let mut reader = stderr;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut tail = tail.lock().unwrap();
                        tail.extend_from_slice(&buf[..n]);
                        let len = tail.len();
                        if len > 8192 {
                            tail.drain(..len - 8192);
                        }
                    }
                }
            }
        });
    }

    if let Ok(mut guard) = HOST_CHILD.get_or_init(|| Mutex::new(None)).lock() {
        *guard = Some(child);
    }

    Ok(())
}

/// Probe the host with a HEAD request: ready only when it completes with any
/// HTTP status. Connect failures, resets, and empty replies all read as
/// not-ready, so the webview waits until the server can actually answer.
fn http_serves(addr: &str) -> bool {
    use std::io::{Read, Write};

    let Ok(mut stream) = TcpStream::connect(addr) else {
        return false;
    };
    // One probe every 500 ms; a short timeout keeps the poll snappy.
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
    if stream
        .write_all(b"HEAD / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut buf = [0u8; 128];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => buf.starts_with(b"HTTP/"),
        _ => false,
    }
}

/// Tail of the host's stderr, kept so `wait_until_ready` can quote the reason
/// the host died instead of reporting only "did not become ready".
static ERR_TAIL: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

/// Last ~8 KB of host stderr, lossily decoded for display.
fn host_error_tail() -> String {
    ERR_TAIL
        .get()
        .and_then(|tail| tail.lock().ok())
        .map(|tail| String::from_utf8_lossy(&tail).into_owned())
        .unwrap_or_default()
}

/// Poll the host until it serves HTTP, the child exits, or the deadline
/// passes. A bare TCP connect is NOT enough: the listener binds before the
/// HTTP stack is up, and a webview navigated in that window renders a blank
/// page — the white-screen failure. Probe for a real HTTP response instead.
/// A child that has already exited means the host crashed at boot — return
/// its stderr immediately rather than spinning for the full timeout, which
/// is what left the splash stuck on a "loading" spinner before.
fn wait_until_ready(addr: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if http_serves(addr) {
            return Ok(());
        }
        // The host dying is the common failure (bad credentials, profile
        // corruption); detect it so the error is immediate and quotable.
        if let Ok(mut guard) = HOST_CHILD.get_or_init(|| Mutex::new(None)).lock() {
            if let Some(child) = guard.as_mut() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let tail = host_error_tail();
                        let reason = if tail.trim().is_empty() {
                            String::new()
                        } else {
                            format!("\n--- host stderr (tail) ---\n{tail}")
                        };
                        return Err(format!(
                            "host at {addr} exited during startup with {status}{reason}"
                        ));
                    }
                    Ok(None) => {}
                    Err(err) => eprintln!("[deepseek-desktop] failed to poll host: {err}"),
                }
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "host at {addr} did not become ready within {timeout:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
