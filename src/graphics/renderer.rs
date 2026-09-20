use tiny_skia::{Paint, Pixmap, Rect, Transform};

use super::Color;

pub struct Renderer {
    pixmap: Pixmap,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = Pixmap::new(width, height).expect("failed to create pixmap");

        Self { pixmap }
    }

    pub fn clear(&mut self, color: Color) {
        self.pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
    }

    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let Some(rect) = Rect::from_xywh(x, y, width, height) else {
            return;
        };

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        paint.anti_alias = true;

        self.pixmap
            .fill_rect(rect, &paint, Transform::identity(), None);
    }

    pub fn into_argb8888(self) -> Vec<u8> {
        let mut pixels = self.pixmap.take();

        /*
         * tiny-skia gives us:
         *
         *     R G B A
         *
         * wl_shm ARGB8888 on
         * little-endian machines expects:
         *
         *     B G R A
         *
         * So swap R and B.
         */
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        pixels
    }
}
