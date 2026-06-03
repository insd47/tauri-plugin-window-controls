use serde::{Deserialize, Serialize};

/// Caption-button color tokens for one theme.
///
/// All values are CSS color strings (`#rrggbb`, `#rrggbbaa`, `transparent`,
/// …). Any omitted token falls back to the plugin's built-in default for that
/// theme. The close-button red, on-red white symbol, window-inactive 32% symbol
/// and disabled symbol are fixed/derived by the plugin and not part of this set.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TitleBarColors {
    /// Caption button background at rest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Caption glyph color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Caption button background on hover.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<String>,
    /// Caption button background while pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed: Option<String>,
    /// Caption button background while the window is inactive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inactive: Option<String>,
}
