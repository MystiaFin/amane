use crate::graphics::{Rect, Renderer, font};
use crate::{Color, Size};

use super::Widget;

pub struct Text {
    content: String,
    size: f32,
    color: Color,
    font: Option<String>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: 16.0,
            color: Color::BLACK,
            font: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn font(mut self, family: &str) -> Self {
        self.font = Some(String::from(family));

        self
    }
}

impl Widget for Text {
    fn width(&self) -> Size {
        let font = font::load(self.font.as_deref());

        let mut total = 0.0;

        for letter in self.content.chars() {
            let metrics = font.metrics(letter, self.size);

            total += metrics.advance_width;
        }

        Size::Fixed(total)
    }

    fn height(&self) -> Size {
        let font = font::load(self.font.as_deref());

        let line = font
            .horizontal_line_metrics(self.size)
            .expect("failed to read line metrics");

        // descent is negative, so this adds the part below the baseline
        Size::Fixed(line.ascent - line.descent)
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let font = font::load(self.font.as_deref());

        let line = font
            .horizontal_line_metrics(self.size)
            .expect("failed to read line metrics");

        let baseline = area.y + line.ascent;

        let mut pen_x = area.x;

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
