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

/// Bundle identifier, which names this app's WebKit and cache directories.
const BUNDLE_ID: &str = "com.deepseek.dsh.desktop";

/// Drop the WKWebView's on-disk state when the host version has changed since
/// the last launch, then record the current version.
///
/// The harness serves a Service Worker. After the vendored dsh is upgraded the
/// previous store's SW and module caches no longer match the new assets, and
/// the page renders blank. Purging on version change gives every upgrade a
/// clean slate automatically, while an unchanged version keeps its store
/// (sessions, settings) across restarts.
///
/// Deliberately *not* done via `WebviewWindowBuilder::data_store_identifier`:
/// pointing the webview at a non-default WKWebsiteDataStore stops Tauri's
/// `tauri://` custom protocol from serving the bundled splash at all, which
/// turns the window pure white — a worse failure than the stale cache it was
/// meant to prevent. Purging the default store's directory achieves the same
/// cache-busting with no effect on asset loading.
///
/// Must run before the webview is created, while WebKit has no handle on the
/// directory. Every step is best-effort: a cache we cannot clear is worth a
/// warning, never a failed boot.
pub fn purge_webview_cache_on_upgrade() {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        eprintln!("[deepseek-desktop] HOME unset; skipping webview cache check");
        return;
    };

    let version = host_version();
    let marker = home
        .join("Library/Application Support")
        .join(BUNDLE_ID)
        .join("webview-host-version");

    // An unreadable or absent marker counts as a mismatch: better to purge a
    // cache that was already clean than to leave a poisoned one in place.
    if std::fs::read_to_string(&marker).is_ok_and(|seen| seen.trim() == version) {
        return;
    }

    for dir in [
        home.join("Library/WebKit").join(BUNDLE_ID),
        home.join("Library/Caches").join(BUNDLE_ID),
    ] {
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => eprintln!("[deepseek-desktop] cleared {}", dir.display()),
            // Nothing to clear is the expected case on a fresh install.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => eprintln!(
                "[deepseek-desktop] could not clear {}: {err}",
                dir.display()
            ),
        }
    }

    if let Some(parent) = marker.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!("[deepseek-desktop] could not create {}: {err}", parent.display());
            return;
        }
    }
    // If the marker cannot be written the next launch purges again — wasteful
    // but harmless, and still correct.
    if let Err(err) = std::fs::write(&marker, &version) {
        eprintln!("[deepseek-desktop] could not record host version: {err}");
    }
}

/// The boot command's host version, e.g. `0.1.1-rc.2`, or `unknown`.
/// Read from the vendored `@deepseek-ai/dsh/package.json` next to the
/// launcher, falling back to the system dsh when the override selects it.
fn host_version() -> String {
    let (program, _) = resolve_host_command();
    // Vendored launcher: version sits at a known relative path.
    if let Some(dir) = std::path::Path::new(&program)
        .parent()
        .map(|d| d.join("node_modules/@deepseek-ai/dsh/package.json"))
    {
        if let Ok(text) = std::fs::read_to_string(&dir) {
            if let Some(v) = extract_json_string_field(&text, "version") {
                return v;
            }
        }
    }
    "unknown".to_string()
}

/// Pull one top-level `"key": "value"` string out of a JSON document, without
/// a full JSON parse — package.json shape is trusted enough for a store salt.
fn extract_json_string_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let rest = &rest[colon + 1..];
    let quote = rest.find('"')?;
    let rest = &rest[quote + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Spawn the host and block until its HTTP endpoint accepts connections.
/// If the preferred command fails at boot, fall back to the system `dsh`
/// before giving up — a stale vendored host (version skew against
/// `~/.dsh/.credentials.yaml` or the profile dir) must not brick the app.
pub fn start_and_wait() -> Result<String, String> {
    let (program, args) = resolve_host_command();

    match spawn_and_wait(&program, &args) {
        Ok(()) => return Ok(wait_for_authenticated_url(Duration::from_secs(5))),
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
                    return Ok(wait_for_authenticated_url(Duration::from_secs(5)));
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
/// Both streams are piped and drained on reader threads: stderr surfaces a
/// host that dies at boot, and stdout carries the authenticated URL the
/// webview must navigate to on newer hosts.
fn spawn_host(program: &str, args: &[String]) -> Result<(), String> {
    let mut child = Command::new(program)
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to spawn host `{program}`: {err}"))?;

    if let Some(stderr) = child.stderr.take() {
        drain_ring(stderr, &ERR_TAIL);
    }
    if let Some(stdout) = child.stdout.take() {
        drain_ring(stdout, &STDOUT_TAIL);
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

/// Tail of the host's stdout, kept so the boot thread can recover the
/// authenticated URL (`dsh web: http://…?token=…`) that newer hosts print.
static STDOUT_TAIL: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

/// Drain a child stream into a shared last-bytes ring until EOF, so the shell
/// can inspect a host's stderr (boot failures) and stdout (the printed URL).
fn drain_ring<R: Read + Send + 'static>(
    reader: R,
    ring: &'static OnceLock<Arc<Mutex<Vec<u8>>>>,
) {
    let tail: Arc<Mutex<Vec<u8>>> = ring.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone();
    std::thread::spawn(move || {
        let mut reader = reader;
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

/// Pull the authenticated web URL out of captured host output. Newer hosts
/// print `dsh web: http://127.0.0.1:3080/?token=…` to stdout; the bare
/// `HOST_URL` is rejected with 401 by those hosts, so the webview must follow
/// the exact printed link (token and all). Returns the last such URL, since a
/// fallback re-spawn prints a fresh token after any earlier attempt.
fn parse_web_url(text: &str) -> Option<String> {
    for line in text.lines().rev() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("dsh web: ") else {
            continue;
        };
        let url = rest.split_whitespace().next().unwrap_or_default();
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url.to_string());
        }
    }
    None
}

/// The last authenticated URL seen on the host's stdout, if any.
fn authenticated_host_url() -> Option<String> {
    let stdout = STDOUT_TAIL
        .get()
        .and_then(|tail| tail.lock().ok())
        .map(|tail| String::from_utf8_lossy(&tail).into_owned())
        .unwrap_or_default();
    parse_web_url(&stdout)
}

/// After readiness, wait briefly for the host to print its authenticated URL.
/// The URL is announced asynchronously (it can land a beat after the listener
/// accepts connections), so poll instead of assuming it is already present.
/// Falls back to the bare `HOST_URL` for older hosts that print no token.
fn wait_for_authenticated_url(timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(url) = authenticated_host_url() {
            return url;
        }
        if Instant::now() >= deadline {
            return HOST_URL.to_string();
        }
        std::thread::sleep(Duration::from_millis(200));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The vendored package.json version is extracted without a JSON parser.
    #[test]
    fn extracts_version_from_package_json_shape() {
        let text = r#"{ "name": "@deepseek-ai/dsh", "version": "0.1.1-rc.2", "bin": {} }"#;
        assert_eq!(
            extract_json_string_field(text, "version").as_deref(),
            Some("0.1.1-rc.2")
        );
    }

    /// The authenticated URL is recovered from the host's stdout line.
    #[test]
    fn parses_authenticated_web_url_from_host_stdout() {
        let text = "noise\ndsh web: http://127.0.0.1:3080/?token=abc123 \nmore";
        assert_eq!(
            parse_web_url(text).as_deref(),
            Some("http://127.0.0.1:3080/?token=abc123")
        );
    }

    /// A fallback re-spawn prints a fresh token; the latest line wins.
    #[test]
    fn prefers_last_web_url_when_host_respawns() {
        let text = "dsh web: http://127.0.0.1:3080/?token=stale\ndsh web: http://127.0.0.1:3080/?token=fresh";
        assert_eq!(
            parse_web_url(text).as_deref(),
            Some("http://127.0.0.1:3080/?token=fresh")
        );
    }

    /// Non-URL output yields no URL (older hosts print no token line).
    #[test]
    fn no_web_url_without_host_line() {
        assert_eq!(parse_web_url("some other output"), None);
    }
}
