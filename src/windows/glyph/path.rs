use std::fmt::Write;

use super::outline::Segment;

/// Side of the square viewBox the path is normalized into. The frontend uses
/// the matching constant `viewBox="0 0 100 100"`.
pub(super) const VIEW: f32 = 100.0;

/// Renormalizes outline segments into SVG path data within a fixed `VIEW`×`VIEW`
/// box: uniform scale to fit, centered. Returns just the `d` string (empty for
/// a glyph with no geometry). Pure — no DirectWrite, unit-testable anywhere.
pub(super) fn to_path(segments: &[Segment]) -> String {
    let Some((min_x, min_y, max_x, max_y)) = bounds(segments) else {
        return String::new();
    };

    let width = (max_x - min_x).max(f32::EPSILON);
    let height = (max_y - min_y).max(f32::EPSILON);
    let scale = VIEW / width.max(height);
    let offset_x = (VIEW - width * scale) / 2.0;
    let offset_y = (VIEW - height * scale) / 2.0;

    let map_x = |x: f32| (x - min_x) * scale + offset_x;
    let map_y = |y: f32| (y - min_y) * scale + offset_y;

    let mut d = String::new();
    for segment in segments {
        match *segment {
            Segment::Move(x, y) => {
                let _ = write!(d, "M{:.2} {:.2}", map_x(x), map_y(y));
            }
            Segment::Line(x, y) => {
                let _ = write!(d, "L{:.2} {:.2}", map_x(x), map_y(y));
            }
            Segment::Cubic(x1, y1, x2, y2, x3, y3) => {
                let _ = write!(
                    d,
                    "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                    map_x(x1), map_y(y1), map_x(x2), map_y(y2), map_x(x3), map_y(y3)
                );
            }
            Segment::Close => d.push('Z'),
        }
    }
    d
}

/// Ink bounds `(min_x, min_y, max_x, max_y)` over all segment points, or `None`
/// when there is no geometry.
fn bounds(segments: &[Segment]) -> Option<(f32, f32, f32, f32)> {
    let mut acc: Option<(f32, f32, f32, f32)> = None;
    let mut add = |x: f32, y: f32| {
        acc = Some(match acc {
            Some((nx, ny, xx, xy)) => (nx.min(x), ny.min(y), xx.max(x), xy.max(y)),
            None => (x, y, x, y),
        });
    };

    for segment in segments {
        match *segment {
            Segment::Move(x, y) | Segment::Line(x, y) => add(x, y),
            Segment::Cubic(x1, y1, x2, y2, x3, y3) => {
                add(x1, y1);
                add(x2, y2);
                add(x3, y3);
            }
            Segment::Close => {}
        }
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_outline_is_blank() {
        assert!(to_path(&[]).is_empty());
    }

    #[test]
    fn normalizes_into_view_box() {
        // A triangle filling its ink box maps to the full VIEW box.
        let segments = [
            Segment::Move(10.0, 10.0),
            Segment::Line(20.0, 10.0),
            Segment::Line(20.0, 20.0),
            Segment::Close,
        ];
        assert_eq!(to_path(&segments), "M0.00 0.00L100.00 0.00L100.00 100.00Z");
    }
}
