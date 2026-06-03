//! Glyph geometry: a font codepoint becomes resolution-independent SVG path
//! data. The backend owns the geometry; the frontend owns SVG presentation.
//!
//! Responsibilities are split (SRP):
//!
//! - [`font`]: resolve a font face for a codepoint (Segoe Fluent → MDL2).
//! - [`outline`]: stream the DirectWrite glyph outline into segments.
//! - [`path`]: normalize the segments into a fixed-viewBox SVG path (pure).

mod font;
mod outline;
mod path;

use crate::{Error, Result};

/// Resolves the first character of `text` to SVG path data (`d`) normalized into
/// a `0 0 100 100` viewBox. Falls back from Segoe Fluent Icons to Segoe MDL2.
///
/// Runs only a handful of times per window (and the frontend caches results), so
/// no backend cache is warranted.
pub(crate) fn glyph_path(text: &str) -> Result<String> {
    let codepoint = text
        .chars()
        .next()
        .ok_or_else(|| Error::Glyph("empty glyph text".into()))? as u32;

    let (face, glyph_index) = font::resolve(codepoint)?;
    let segments = outline::extract(&face, glyph_index)?;
    Ok(path::to_path(&segments))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Extracts the real caption glyphs through DirectWrite on the host machine.
    #[test]
    fn extracts_caption_glyphs() {
        for (name, glyph) in [
            ("minimize", "\u{E921}"),
            ("maximize", "\u{E922}"),
            ("restore", "\u{E923}"),
            ("close", "\u{E8BB}"),
        ] {
            let d = glyph_path(glyph).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!d.is_empty(), "{name} produced no path data");
            assert!(d.starts_with('M'), "{name}: {d}");
        }
    }
}
