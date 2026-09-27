use vello::kurbo::{Affine, BezPath};
use vello::peniko;

use crate::graphics::path::Segment;
use crate::graphics::{Color, Path, Transform};

pub(super) fn bezier(path: &Path) -> BezPath {
    let mut bezier = BezPath::new();

    for segment in &path.segments {
        match *segment {
            Segment::MoveTo(x, y) => bezier.move_to(point(x, y)),

            Segment::LineTo(x, y) => bezier.line_to(point(x, y)),

            Segment::QuadTo(x1, y1, x, y) => bezier.quad_to(point(x1, y1), point(x, y)),

            Segment::CubicTo(x1, y1, x2, y2, x, y) => {
                bezier.curve_to(point(x1, y1), point(x2, y2), point(x, y))
            }

            Segment::Close => bezier.close_path(),
        }
    }

    bezier
}

pub(super) fn affine(transform: Transform) -> Affine {
    Affine::new([
        f64::from(transform.sx),
        f64::from(transform.ky),
        f64::from(transform.kx),
        f64::from(transform.sy),
        f64::from(transform.tx),
        f64::from(transform.ty),
    ])
}

pub(super) fn paint(color: Color) -> peniko::Color {
    peniko::Color::from_rgba8(color.r, color.g, color.b, color.a)
}

fn point(x: f32, y: f32) -> (f64, f64) {
    (f64::from(x), f64::from(y))
}
