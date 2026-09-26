use tiny_skia::{FillRule, Paint, Stroke};

use crate::graphics::{Color, Rect, Renderer};

impl Renderer {
    pub fn rectangle(
        &mut self,
        rect: Rect,
        color: Color,
        radius: f32,
        border_thickness: f32,
        border_color: Color,
    ) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        self.pixmap
            .fill_path(&path, &paint, FillRule::Winding, self.transform, None);

        // a zero width line would still draw as a hairline
        if border_thickness == 0.0 {
            return;
        }

        /*
         * a line is drawn centered on its path,
         * so pulling the path in by half the thickness keeps the line inside the edge
         */
        let half_thickness = border_thickness / 2.0;

        let border_rect = Rect::new(
            rect.x + half_thickness,
            rect.y + half_thickness,
            rect.width - border_thickness,
            rect.height - border_thickness,
        );

        // the pulled in path curves tighter, but never past a square corner
        let border_radius = f32::max(radius - half_thickness, 0.0);

        let Some(border_path) = border_rect.trace(border_radius) else {
            return;
        };

        let mut border_paint = Paint::default();

        border_paint.set_color_rgba8(
            border_color.r,
            border_color.g,
            border_color.b,
            border_color.a,
        );

        let stroke = Stroke {
            width: border_thickness,
            ..Stroke::default()
        };

        self.pixmap
            .stroke_path(&border_path, &border_paint, &stroke, self.transform, None);
    }
}
