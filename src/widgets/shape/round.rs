use crate::Size;

use super::Style;

// where a circle or arc sits: its center and radius in canvas coordinates
pub fn place(
    center: Option<(f32, f32)>,
    radius: Size,
    style: &Style,
    width: f32,
    height: f32,
) -> (f32, f32, f32) {
    // no center means the middle of the canvas
    let (center_x, center_y) = center.unwrap_or((width / 2.0, height / 2.0));

    /*
     * a line is drawn centered on its path,
     * so a radius that fills the canvas pulls in by half the line to keep it inside
     */
    let half_thickness = style.stroke_thickness / 2.0;
    let fitting_radius = f32::min(width, height) / 2.0 - half_thickness;

    let radius = radius.resolve(fitting_radius);

    (center_x, center_y, radius)
}
