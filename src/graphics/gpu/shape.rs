use vello::Scene;
use vello::kurbo::{Join, Stroke};
use vello::peniko::Fill;

use crate::graphics::{Color, Path, Transform};

use super::convert::{affine, bezier, paint};

pub(super) fn fill(scene: &mut Scene, path: &Path, transform: Transform, color: Color) {
    scene.fill(
        Fill::NonZero,
        affine(transform),
        paint(color),
        None,
        &bezier(path),
    );
}

pub(super) fn stroke(
    scene: &mut Scene,
    path: &Path,
    transform: Transform,
    thickness: f32,
    color: Color,
) {
    // square corners stay square instead of being rounded off by the line
    let stroke = Stroke::new(f64::from(thickness)).with_join(Join::Miter);

    scene.stroke(
        &stroke,
        affine(transform),
        paint(color),
        None,
        &bezier(path),
    );
}
