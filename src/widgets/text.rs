use crate::Color;
use crate::graphics::{Renderer, font};

use super::Widget;

pub struct Text {
    pub content: String,
    pub size: f32,
    pub color: Color,
    pub font: Option<String>,
}

impl Widget for Text {
    fn width(&self) -> f32 {
        let font = font::load(self.font.as_deref());

        let mut total = 0.0;

        for letter in self.content.chars() {
            let metrics = font.metrics(letter, self.size);

            total += metrics.advance_width;
        }

        total
    }

    fn height(&self) -> f32 {
        let font = font::load(self.font.as_deref());

        let line = font
            .horizontal_line_metrics(self.size)
            .expect("failed to read line metrics");

        // descent is negative, so this adds the part below the baseline
        line.ascent - line.descent
    }

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32) {
        let font = font::load(self.font.as_deref());

        let line = font
            .horizontal_line_metrics(self.size)
            .expect("failed to read line metrics");

        let baseline = y + line.ascent;

        let mut pen_x = x;

        for letter in self.content.chars() {
            let (metrics, coverage) = font.rasterize(letter, self.size);

            /*
             * fontdue measures the letter from the baseline upward,
             * the renderer wants its top-left corner
             */
            let left = pen_x + metrics.xmin as f32;
            let top = baseline - (metrics.ymin as f32 + metrics.height as f32);

            renderer.glyph(
                left.round() as i32,
                top.round() as i32,
                metrics.width as u32,
                metrics.height as u32,
                &coverage,
                self.color,
            );

            pen_x += metrics.advance_width;
        }
    }
}
