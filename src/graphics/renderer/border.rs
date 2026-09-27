use tiny_skia::{Paint, Stroke};

use crate::graphics::{Color, Rect, Renderer};

impl Renderer {
    pub fn border(&mut self, rect: Rect, radius: f32, thickness: f32, color: Color) {
        // a zero width line would still draw as a hairline
        if thickness == 0.0 {
            return;
        }

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

        let Some(border_path) = border_rect.trace(border_radius) else {
            return;
        };

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        let stroke = Stroke {
            width: thickness,
            ..Stroke::default()
        };

        self.pixmap
            .stroke_path(&border_path, &paint, &stroke, self.transform, None);
    }
}
