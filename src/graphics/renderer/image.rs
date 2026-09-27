use tiny_skia::{FillRule, FilterQuality, Mask, Pixmap, PixmapPaint, Transform};

use crate::graphics::{Rect, Renderer};

impl Renderer {
    pub fn image(&mut self, rect: Rect, radius: f32, image: &Pixmap, placement: Rect) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        // the image can reach past the rectangle, the mask keeps it inside the rounded shape
        let mut shape =
            Mask::new(self.pixmap.width(), self.pixmap.height()).expect("failed to create mask");

        shape.fill_path(&path, FillRule::Winding, true, self.transform);

        let horizontal_scale = placement.width / image.width() as f32;
        let vertical_scale = placement.height / image.height() as f32;

        let image_transform = Transform::from_row(
            horizontal_scale,
            0.0,
            0.0,
            vertical_scale,
            placement.x,
            placement.y,
        );

        let transform = image_transform.post_concat(self.transform);

        // bicubic keeps a large image smooth when it shrinks to fit
        let paint = PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };

        self.pixmap
            .draw_pixmap(0, 0, image.as_ref(), &paint, transform, Some(&shape));
    }
}
