use vello::Scene;
use vello::kurbo::{self, Join, Stroke};
use vello::peniko::Fill;

use crate::graphics::{BezierPath, Cap, Color, Rect, Transform};

use super::convert::{affine, bezier, paint};

pub(super) fn fill(scene: &mut Scene, path: &BezierPath, transform: Transform, color: Color) {
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
    path: &BezierPath,
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

// the outline route for a rounded rectangle, when the quads can't draw it
pub(super) fn rectangle(
    scene: &mut Scene,
    rect: Rect,
    radius: f32,
    transform: Transform,
    color: Color,
) {
    let Some(path) = rect.trace(radius) else {
        return;
    };

    fill(scene, &path, transform, color);
}

pub(super) fn border(
    scene: &mut Scene,
    rect: Rect,
    radius: f32,
    thickness: f32,
    transform: Transform,
    color: Color,
) {
    /*
     * a line is drawn centered on its path,
     * so pulling the path in by half the thickness keeps the line inside the edge
     */
    let half_thickness = thickness / 2.0;

    let border_rect = Rect::new(
        rect.x + half_thickness,
        rect.y + half_thickness,
        rect.width - thickness,
        rect.height - thickness,
    );

    // the pulled in path curves tighter, but never past a square corner
    let border_radius = f32::max(radius - half_thickness, 0.0);

    let Some(path) = border_rect.trace(border_radius) else {
        return;
    };

    stroke(scene, &path, transform, thickness, color, Cap::Butt);
}
