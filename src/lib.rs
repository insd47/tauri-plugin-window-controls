//! Native Windows title bar overlay for Tauri windows.
//!
//! Windows-only by design: on every other target the extension-trait methods
//! are no-ops and [`init`] registers an empty plugin, so nothing from this
//! crate ends up in non-Windows builds.

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod error;
mod ext;
mod models;

#[cfg(windows)]
mod commands;
#[cfg(windows)]
mod windows;

pub use error::{Error, Result};
pub use ext::{WindowControlsBuilderExt, WindowControlsExt};
pub use models::TitleBarColors;

/// Initializes the plugin. Register it with `tauri::Builder::plugin(init())`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::new("window-controls");

    #[cfg(windows)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::get_glyph_path,
        commands::window_command,
        commands::window_state,
        commands::enable_snap
    ]);

    builder.build()
}
