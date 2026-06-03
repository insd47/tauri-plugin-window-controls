use windows::{
    core::PCWSTR,
    Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, IDWriteFontFace,
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_NORMAL,
    },
};
use windows_core::BOOL;

use crate::{Error, Result};

/// Preferred icon font first, then the legacy fallback so the same glyph
/// codepoints render across Windows 10/11 with minimal visual drift.
const FAMILIES: [&str; 2] = ["Segoe Fluent Icons", "Segoe MDL2 Assets"];

/// Resolves a font face that actually contains a glyph for `codepoint`, via the
/// DirectWrite system font collection. Returns the face and a non-zero glyph
/// index.
pub(super) fn resolve(codepoint: u32) -> Result<(IDWriteFontFace, u16)> {
    unsafe {
        let factory: IDWriteFactory =
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).map_err(win_err)?;

        let mut collection: Option<IDWriteFontCollection> = None;
        factory
            .GetSystemFontCollection(&mut collection, false)
            .map_err(win_err)?;
        let collection =
            collection.ok_or_else(|| Error::Glyph("system font collection unavailable".into()))?;

        for family in FAMILIES {
            let Some(face) = create_face(&collection, family)? else {
                continue;
            };

            let mut index: u16 = 0;
            face.GetGlyphIndices(&codepoint, 1, &mut index)
                .map_err(win_err)?;

            if index != 0 {
                return Ok((face, index));
            }
        }

        Err(Error::Glyph(format!("no glyph found for U+{codepoint:04X}")))
    }
}

unsafe fn create_face(
    collection: &IDWriteFontCollection,
    family: &str,
) -> Result<Option<IDWriteFontFace>> {
    let name: Vec<u16> = family.encode_utf16().chain(std::iter::once(0)).collect();
    let mut index = 0u32;
    let mut exists = BOOL(0);

    collection
        .FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists)
        .map_err(win_err)?;

    if !exists.as_bool() {
        return Ok(None);
    }

    let family = collection.GetFontFamily(index).map_err(win_err)?;
    let font = family
        .GetFirstMatchingFont(
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
        )
        .map_err(win_err)?;

    Ok(Some(font.CreateFontFace().map_err(win_err)?))
}

fn win_err(error: windows::core::Error) -> Error {
    Error::Glyph(error.to_string())
}
