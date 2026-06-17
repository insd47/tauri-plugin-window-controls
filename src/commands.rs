//! Plugin commands. All are Windows-only — on other targets none are defined
//! and `init` registers no handlers, so the plugin leaves no trace.

use crate::Result;
use serde::Serialize;
use tauri::{command, Runtime, WebviewWindow};
use tauri_plugin_system_symbols::Path;

/// Returns SVG paths for a single platform symbol. The frontend wraps them in
/// `<svg>` and preserves per-path fill rules / opacity.
#[command]
pub(crate) async fn get_glyph_path(text: String) -> Result<Vec<Path>> {
    Ok(tauri_plugin_system_symbols::get_symbol(text, 10.0)?)
}

/// Performs a caption action on the invoking window.
#[command]
pub(crate) async fn window_command<R: Runtime>(
    window: WebviewWindow<R>,
    action: String,
) -> Result<()> {
    match action.as_str() {
        "minimize" => window.minimize()?,
        "toggle-maximize" => {
            if window.is_maximized()? {
                window.unmaximize()?;
            } else {
                window.maximize()?;
            }
        }
        "close" => window.close()?,
        _ => {}
    }
    Ok(())
}

/// Current caption-relevant window state, used by the injected runtime to
/// render the right glyph / enabled / active styling.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WindowState {
    maximized: bool,
    focused: bool,
    minimizable: bool,
    maximizable: bool,
    closable: bool,
}

/// Installs the Windows 11 snap-layout overlay over the maximize button.
/// Called by the injected runtime once the caption bar is built.
#[command]
pub(crate) async fn enable_snap<R: Runtime>(window: WebviewWindow<R>, height: u32) -> Result<()> {
    crate::windows::overlay::snap::install(&window, height)?;
    Ok(())
}

#[command]
pub(crate) async fn window_state<R: Runtime>(window: WebviewWindow<R>) -> Result<WindowState> {
    Ok(WindowState {
        maximized: window.is_maximized()?,
        focused: window.is_focused()?,
        minimizable: window.is_minimizable()?,
        maximizable: window.is_maximizable()?,
        closable: window.is_closable()?,
    })
}
