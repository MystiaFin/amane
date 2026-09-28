use vello::Scene;
use vello::kurbo::{self, Join, Stroke};
use vello::peniko::Fill;

use crate::graphics::{Cap, Color, Path, Transform};

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
    cap: Cap,
) {
    // square corners stay square instead of being rounded off by the line
    let stroke = Stroke::new(f64::from(thickness))
        .with_join(Join::Miter)
        .with_caps(line_end(cap));

    scene.stroke(
        &stroke,
        affine(transform),
        paint(color),
        None,
        &bezier(path),
    );
}

fn line_end(cap: Cap) -> kurbo::Cap {
    match cap {
        Cap::Butt => kurbo::Cap::Butt,
        Cap::Round => kurbo::Cap::Round,
        Cap::Square => kurbo::Cap::Square,
    }
}
