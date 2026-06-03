//! Public DX: builder- and window-level extension traits.
//!
//! Mirrors how macOS title-bar tweaks are applied through the Tauri window
//! builder, so the same code reads naturally cross-platform. Every method is a
//! **no-op on non-Windows** targets — the plugin leaves no runtime trace there.

use tauri::{Manager, Runtime, WebviewWindow, WebviewWindowBuilder};

use crate::models::TitleBarColors;

/// Configure the native title bar overlay while building a window.
pub trait WindowControlsBuilderExt: Sized {
    /// Enables the overlay: removes the system frame, keeps the drop shadow,
    /// and injects the caption-control runtime. Default is disabled.
    fn set_title_bar_overlay(self, enabled: bool) -> Self;
    /// Sets the title bar / caption button height in logical pixels (default 32).
    fn set_title_bar_height(self, height: u32) -> Self;
    /// Overrides the caption colors per theme (rarely needed — sensible
    /// Windows-native defaults are built in).
    fn set_title_bar_colors(self, light: TitleBarColors, dark: TitleBarColors) -> Self;
}

impl<'a, R: Runtime, M: Manager<R>> WindowControlsBuilderExt for WebviewWindowBuilder<'a, R, M> {
    fn set_title_bar_overlay(self, enabled: bool) -> Self {
        #[cfg(windows)]
        {
            if enabled {
                return self
                    .decorations(false)
                    .shadow(true)
                    .initialization_script(crate::windows::overlay::BOOTSTRAP_JS);
            }
            self
        }
        #[cfg(not(windows))]
        {
            let _ = enabled;
            self
        }
    }

    fn set_title_bar_height(self, height: u32) -> Self {
        #[cfg(windows)]
        {
            self.initialization_script(&crate::windows::overlay::height_script(height))
        }
        #[cfg(not(windows))]
        {
            let _ = height;
            self
        }
    }

    fn set_title_bar_colors(self, light: TitleBarColors, dark: TitleBarColors) -> Self {
        #[cfg(windows)]
        {
            self.initialization_script(&crate::windows::overlay::colors_script(&light, &dark))
        }
        #[cfg(not(windows))]
        {
            let _ = (light, dark);
            self
        }
    }
}

/// Configure the native title bar overlay on an already-created window.
///
/// Useful for windows declared in `tauri.conf.json`. Persistent injection
/// happens at build time; on an existing window the runtime is injected via a
/// one-shot `eval`, which suits single-page apps (no full reloads).
pub trait WindowControlsExt {
    fn set_title_bar_overlay(&self, enabled: bool) -> tauri::Result<()>;
    fn set_title_bar_height(&self, height: u32) -> tauri::Result<()>;
    fn set_title_bar_colors(&self, light: TitleBarColors, dark: TitleBarColors) -> tauri::Result<()>;
}

impl<R: Runtime> WindowControlsExt for WebviewWindow<R> {
    fn set_title_bar_overlay(&self, enabled: bool) -> tauri::Result<()> {
        #[cfg(windows)]
        {
            if enabled {
                self.set_decorations(false)?;
                let _ = self.set_shadow(true);
                self.eval(crate::windows::overlay::BOOTSTRAP_JS)?;
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = enabled;
            Ok(())
        }
    }

    fn set_title_bar_height(&self, height: u32) -> tauri::Result<()> {
        #[cfg(windows)]
        {
            self.eval(&crate::windows::overlay::height_script(height))
        }
        #[cfg(not(windows))]
        {
            let _ = height;
            Ok(())
        }
    }

    fn set_title_bar_colors(&self, light: TitleBarColors, dark: TitleBarColors) -> tauri::Result<()> {
        #[cfg(windows)]
        {
            self.eval(&crate::windows::overlay::colors_script(&light, &dark))
        }
        #[cfg(not(windows))]
        {
            let _ = (light, dark);
            Ok(())
        }
    }
}
