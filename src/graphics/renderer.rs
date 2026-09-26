mod rectangle;
mod text;

use tiny_skia::{Pixmap, Transform};

use super::Color;

pub struct Renderer {
    pixmap: Pixmap,
    transform: Transform,
}

impl Renderer {
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        let pixmap = Pixmap::new(width, height).expect("failed to create pixmap");

        // widgets measure in logical pixels, the pixmap in real ones
        let horizontal_scale = scale;
        let vertical_scale = scale;

        let transform = Transform::from_scale(horizontal_scale, vertical_scale);

        Self { pixmap, transform }
    }

    pub fn clear(&mut self, color: Color) {
        self.pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
    }

    pub fn into_argb8888(self) -> Vec<u8> {
        let mut pixels = self.pixmap.take();

        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        pixels
    }
}
