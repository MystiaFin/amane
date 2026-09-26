use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::graphics::Renderer;

impl Renderer {
    pub fn layer(&self) -> Self {
        let pixmap = Pixmap::new(self.pixmap.width(), self.pixmap.height())
            .expect("failed to create pixmap");

        Self {
            pixmap,
            transform: self.transform,
        }
    }

    pub fn blend(&mut self, layer: Renderer, opacity: f32) {
        let paint = PixmapPaint {
            opacity,
            ..PixmapPaint::default()
        };

        self.pixmap.draw_pixmap(
            0,
            0,
            layer.pixmap.as_ref(),
            &paint,
            Transform::identity(),
            None,
        );
    }
}
