//! Injected runtime and config scripts for the auto-injected caption controls.
//!
//! The builder/window extension traits add these as initialization scripts. The
//! config setters only assign globals; [`BOOTSTRAP_JS`] reads them on DOM ready
//! (with built-in defaults), so the three setters are order-independent.

use crate::models::TitleBarColors;

pub(crate) mod snap;

/// Framework-agnostic vanilla runtime that renders the caption controls.
pub(crate) const BOOTSTRAP_JS: &str = include_str!("controls.js");

/// Sets the title bar height global (logical px).
pub(crate) fn height_script(height: u32) -> String {
    format!("window.__TBO_HEIGHT__={height};")
}

/// Sets the per-theme color override global.
pub(crate) fn colors_script(light: &TitleBarColors, dark: &TitleBarColors) -> String {
    let light = serde_json::to_string(light).unwrap_or_else(|_| "{}".into());
    let dark = serde_json::to_string(dark).unwrap_or_else(|_| "{}".into());
    format!("window.__TBO_COLORS__={{\"light\":{light},\"dark\":{dark}}};")
}
