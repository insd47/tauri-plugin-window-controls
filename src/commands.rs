//! Plugin commands. All are Windows-only — on other targets none are defined
//! and `init` registers no handlers, so the plugin leaves no trace.

#[cfg(windows)]
use serde::Serialize;
#[cfg(windows)]
use tauri::{command, Runtime, WebviewWindow};

/// Returns SVG path data for a single glyph character (Segoe Fluent Icons,
/// falling back to Segoe MDL2 Assets). The frontend wraps it in `<svg>`.
#[cfg(windows)]
#[command]
pub(crate) async fn get_glyph_path(text: String) -> crate::Result<String> {
    crate::windows::glyph_path(&text)
}

/// Performs a caption action on the invoking window.
#[cfg(windows)]
#[command]
pub(crate) async fn window_command<R: Runtime>(
    window: WebviewWindow<R>,
    action: String,
) -> crate::Result<()> {
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
#[cfg(windows)]
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
#[cfg(windows)]
#[command]
pub(crate) async fn enable_snap<R: Runtime>(
    window: WebviewWindow<R>,
    height: u32,
) -> crate::Result<()> {
    crate::windows::overlay::snap::install(&window, height)?;
    Ok(())
}

#[cfg(windows)]
#[command]
pub(crate) async fn window_state<R: Runtime>(
    window: WebviewWindow<R>,
) -> crate::Result<WindowState> {
    Ok(WindowState {
        maximized: window.is_maximized()?,
        focused: window.is_focused()?,
        minimizable: window.is_minimizable()?,
        maximizable: window.is_maximizable()?,
        closable: window.is_closable()?,
    })
}
