//! Desktop shell wiring: tray, global shortcut, and host boot.
//!
//! First-principles split: this module only glues the OS shell to the official
//! DeepSeek Harness host. It never reimplements the harness, the agent loop, or
//! the web UI — those all live in the upstream `@deepseek-ai/dsh` the host runs.

pub mod host;
pub mod tray;
pub mod updater;

use tauri::{AppHandle, Manager};

/// Wire up the tray, the global toggle shortcut, and the harness host.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    tray::build(app)?;
    register_global_shortcut(app)?;

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

/// Toggle the main window on a global Cmd/Ctrl+Shift+D press.
fn register_global_shortcut(app: &AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
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
