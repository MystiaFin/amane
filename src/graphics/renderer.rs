use tiny_skia::{ColorU8, Paint, Pixmap, PixmapPaint, Rect as SkiaRect, Transform};

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

    pub fn rectangle(&mut self, rect: Rect, color: Color) {
        let Some(rect) = SkiaRect::from_xywh(rect.x, rect.y, rect.width, rect.height) else {
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
