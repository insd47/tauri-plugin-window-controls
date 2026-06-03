use std::{cell::RefCell, rc::Rc};

use windows::{
    core::implement,
    Win32::Graphics::{
        Direct2D::Common::{
            ID2D1SimplifiedGeometrySink, ID2D1SimplifiedGeometrySink_Impl, D2D1_BEZIER_SEGMENT,
            D2D1_FIGURE_BEGIN, D2D1_FIGURE_END, D2D1_FILL_MODE, D2D1_PATH_SEGMENT,
        },
        DirectWrite::IDWriteFontFace,
    },
};
use windows_numerics::Vector2;

use crate::{Error, Result};

/// Logical em size requested from DirectWrite. Coordinates are renormalized
/// into a fixed viewBox later, so this only fixes precision.
const EM: f32 = 1000.0;

/// One SVG path command, in the glyph's own (y-down) coordinate space.
pub(super) enum Segment {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

/// Extracts the vector outline for a glyph via DirectWrite as path segments.
///
/// `GetGlyphRunOutline` streams the contour into a geometry sink; the output
/// already uses a y-down axis, matching SVG. Bounds/normalization are the
/// `path` module's job.
pub(super) fn extract(face: &IDWriteFontFace, glyph_index: u16) -> Result<Vec<Segment>> {
    let segments = Rc::new(RefCell::new(Vec::new()));
    let sink: ID2D1SimplifiedGeometrySink = PathSink {
        segments: segments.clone(),
    }
    .into();

    unsafe {
        face.GetGlyphRunOutline(EM, &glyph_index, None, None, 1, false, false, &sink)
            .map_err(|error| Error::Glyph(error.to_string()))?;
    }

    Ok(segments.take())
}

/// COM geometry sink that records the streamed contour as path segments.
#[implement(ID2D1SimplifiedGeometrySink)]
struct PathSink {
    segments: Rc<RefCell<Vec<Segment>>>,
}

// COM interface method names are fixed PascalCase (foreign ABI) — can't rename.
#[allow(non_snake_case)]
impl ID2D1SimplifiedGeometrySink_Impl for PathSink_Impl {
    fn SetFillMode(&self, _fillmode: D2D1_FILL_MODE) {}

    fn SetSegmentFlags(&self, _flags: D2D1_PATH_SEGMENT) {}

    fn BeginFigure(&self, start: &Vector2, _begin: D2D1_FIGURE_BEGIN) {
        self.segments.borrow_mut().push(Segment::Move(start.X, start.Y));
    }

    fn AddLines(&self, points: *const Vector2, count: u32) {
        let points = unsafe { std::slice::from_raw_parts(points, count as usize) };
        let mut segments = self.segments.borrow_mut();
        for point in points {
            segments.push(Segment::Line(point.X, point.Y));
        }
    }

    fn AddBeziers(&self, beziers: *const D2D1_BEZIER_SEGMENT, count: u32) {
        let beziers = unsafe { std::slice::from_raw_parts(beziers, count as usize) };
        let mut segments = self.segments.borrow_mut();
        for b in beziers {
            segments.push(Segment::Cubic(
                b.point1.X, b.point1.Y, b.point2.X, b.point2.Y, b.point3.X, b.point3.Y,
            ));
        }
    }

    fn EndFigure(&self, _end: D2D1_FIGURE_END) {
        self.segments.borrow_mut().push(Segment::Close);
    }

    fn Close(&self) -> windows_core::Result<()> {
        Ok(())
    }
}
