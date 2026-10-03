use vello::Scene;
use vello::kurbo::Point;
use vello::peniko::{self, Fill};

use crate::graphics::{BezierPath, Color, Gradient, Area, Transform};

use super::convert::{affine, bezier, paint};

pub(super) fn fill(
    scene: &mut Scene,
    path: &BezierPath,
    area: Area,
    transform: Transform,
    gradient: &Gradient,
) {
    let (shape, stops) = match gradient {
        Gradient::Linear { angle, stops } => (linear(area, *angle), stops),
        Gradient::Radial { stops } => (radial(area), stops),
    };

    let brush = shape.with_stops(colors(stops).as_slice());

    scene.fill(
        Fill::NonZero,
        affine(transform),
        &brush,
        None,
        &bezier(path),
    );
}

// a line through the center, long enough that the first and last stops touch the corners
fn linear(area: Area, angle: f32) -> peniko::Gradient {
    let (sin, cos) = angle.to_radians().sin_cos();

    let half_length = (area.width * sin.abs() + area.height * cos.abs()) / 2.0;

    let center_x = area.x + area.width / 2.0;
    let center_y = area.y + area.height / 2.0;

    // y grows downward, so 0 degrees pointing up means going toward smaller y
    let step_x = sin * half_length;
    let step_y = -cos * half_length;

    let start = point(center_x - step_x, center_y - step_y);
    let end = point(center_x + step_x, center_y + step_y);

    peniko::Gradient::new_linear(start, end)
}

fn radial(area: Area) -> peniko::Gradient {
    let center = point(area.x + area.width / 2.0, area.y + area.height / 2.0);

    let radius = area.width.hypot(area.height) / 2.0;

    peniko::Gradient::new_radial(center, radius)
}

fn colors(stops: &[(f32, Color)]) -> Vec<(f32, peniko::Color)> {
    stops
        .iter()
        .map(|&(offset, color)| (offset, paint(color)))
        .collect()
}

fn point(x: f32, y: f32) -> Point {
    Point::new(f64::from(x), f64::from(y))
}
