use tiny_skia::{ColorU8, Pixmap, PixmapPaint, Transform};

use super::{Color, Rect};

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

    pub fn rectangle(&mut self, rect: Rect, color: Color, radius: f32) {
        let left = rect.x.floor() as i32;
        let top = rect.y.floor() as i32;
        let right = (rect.x + rect.width).ceil() as i32;
        let bottom = (rect.y + rect.height).ceil() as i32;

        let width = i32::max(right - left, 0) as u32;
        let height = i32::max(bottom - top, 0) as u32;

        if width == 0 || height == 0 {
            return;
        }

        let mut coverage = Vec::with_capacity((width * height) as usize);

        for row in 0..height {
            for column in 0..width {
                // sample the middle of the pixel, not its top-left corner
                let x = left as f32 + column as f32 + 0.5;
                let y = top as f32 + row as f32 + 0.5;

                let distance = rect.distance(x, y, radius);

                // pixels within half a pixel of the edge are partly covered
                let fraction = (0.5 - distance).clamp(0.0, 1.0);

                let amount = (fraction * 255.0).round() as u8;

                coverage.push(amount);
            }
        }

        self.glyph(left, top, width, height, &coverage, color);
    }

    pub fn into_argb8888(self) -> Vec<u8> {
        let mut pixels = self.pixmap.take();

        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        pixels
    }

    pub fn glyph(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        coverage: &[u8],
        color: Color,
    ) {
        let Some(mut glyph) = Pixmap::new(width, height) else {
            return;
        };

        for (pixel, &amount) in glyph.pixels_mut().iter_mut().zip(coverage) {
            let color_alpha = u16::from(color.a);
            let coverage = u16::from(amount);
            let alpha = (color_alpha * coverage / 255) as u8;

            *pixel = ColorU8::from_rgba(color.r, color.g, color.b, alpha).premultiply();
        }

        self.pixmap.draw_pixmap(
            x,
            y,
            glyph.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
}
