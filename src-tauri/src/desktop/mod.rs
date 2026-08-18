//! Desktop shell wiring: tray, global shortcut, host boot, and the main window.
//!
//! First-principles split: this module only glues the OS shell to the official
//! DeepSeek Harness host. It never reimplements the harness, the agent loop, or
//! the web UI — those all live in the upstream `@deepseek-ai/dsh` the host runs.

pub mod host;
pub mod tray;
pub mod updater;

use tauri::{AppHandle, Manager};

/// Wire up the tray, the global toggle shortcut, the main window, and the host.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    tray::build(app)?;

    // The global shortcut is a convenience, not a hard requirement: log and
    // continue if its registration fails (e.g. the hotkey is already taken).
    if let Err(err) = register_global_shortcut(app) {
        eprintln!("[deepseek-desktop] global shortcut unavailable: {err}");
    }

    // Create the main window with external-link handling (internal URLs navigate
    // in-place; everything else opens in the system browser).
    create_main_window(app)?;

    // Boot the host on a dedicated thread (it blocks on a TCP readiness poll),
    // then navigate the webview to the harness UI once it answers.
    let handle = app.clone();
    std::thread::spawn(move || match host::start_and_wait() {
        Ok(url) => {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.eval(&format!("window.location.replace('{url}')"));
            }
        }
        Err(err) => eprintln!("[deepseek-desktop] {err}"),
    });

    Ok(())
}

/// Create the main window, which loads the static splash and is later navigated
/// to the harness UI by the host-boot thread. Links to the local host navigate
/// in-place; any other URL (and `target="_blank"` requests) open in the user's
/// default browser instead of taking over the webview.
fn create_main_window(app: &AppHandle) -> tauri::Result<()> {
    tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App(std::path::PathBuf::from("index.html")),
    )
    .title("DeepSeek Harness Desktop")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .on_navigation(|url| {
        if matches!(url.host_str(), Some("127.0.0.1") | Some("localhost")) {
            true
        } else {
            open_in_browser(url.as_str());
            false
        }
    })
    .on_new_window(|url, _features| {
        open_in_browser(url.as_str());
        tauri::webview::NewWindowResponse::Deny
    })
    .build()?;

    Ok(())
}

/// Open a URL in the user's default browser via the OS opener.
fn open_in_browser(url: &str) {
    let spawned = {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(url).spawn()
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/c", "start", "", url])
                .spawn()
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open").arg(url).spawn()
        }
    };

    if let Err(err) = spawned {
        eprintln!("[deepseek-desktop] failed to open external URL: {err}");
    }
}

/// Toggle the main window on a global Cmd/Ctrl+Shift+D press.
fn register_global_shortcut(app: &AppHandle) -> Result<(), tauri_plugin_global_shortcut::Error> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })?;

    Ok(())
}
