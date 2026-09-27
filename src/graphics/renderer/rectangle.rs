use tiny_skia::{FillRule, Paint};

use crate::graphics::{Color, Rect, Renderer};

impl Renderer {
    pub fn rectangle(&mut self, rect: Rect, color: Color, radius: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        self.pixmap
            .fill_path(&path, &paint, FillRule::Winding, self.transform, None);
    }
}
