use vello::Scene;
use vello::kurbo;
use vello::peniko::{self, Compose, Fill, Mix};

use crate::graphics::{Area, BezierPath, Color, Corners, Transform};

use super::convert::{affine, bezier, paint};

pub(super) fn drop_shadow(
    scene: &mut Scene,
    area: Area,
    radius: Corners,
    transform: Transform,
    color: Color,
    blur: f32,
) {
    // ponytail: vello blurs one radius, so shadows round every corner like the largest; trace a blurred path if mixed corners show
    scene.draw_blurred_rounded_rect(
        affine(transform),
        rounded(area),
        paint(color),
        f64::from(radius.largest()),
        deviation(blur),
    );
}

pub(super) fn inner_shadow(
    scene: &mut Scene,
    clip: &BezierPath,
    hole: Area,
    radius: Corners,
    transform: Transform,
    color: Color,
    blur: f32,
) {
    let transform = affine(transform);

    let clip = bezier(clip);

    // a layer of its own, so cutting the hole only reaches the shadow and not the fill
    scene.push_layer(Fill::NonZero, Mix::Normal, 1.0, transform, &clip);

    scene.fill(Fill::NonZero, transform, paint(color), None, &clip);

    // whatever is drawn in this layer is taken away from the shadow instead of added
    scene.push_layer(Fill::NonZero, Compose::DestOut, 1.0, transform, &clip);

    scene.draw_blurred_rounded_rect(
        transform,
        rounded(hole),
        peniko::Color::BLACK,
        f64::from(radius.largest()),
        deviation(blur),
    );

    scene.pop_layer();

    scene.pop_layer();
}

fn rounded(area: Area) -> kurbo::Rect {
    let right = area.x + area.width;
    let bottom = area.y + area.height;

    kurbo::Rect::new(
        f64::from(area.x),
        f64::from(area.y),
        f64::from(right),
        f64::from(bottom),
    )
}

// like css, a shadow's blur reaches twice as far as the spread of the gaussian behind it
fn deviation(blur: f32) -> f64 {
    f64::from(blur) / 2.0
}
