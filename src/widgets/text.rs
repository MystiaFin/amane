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

        // fonts measure in their own units, this turns them into pixels
        let units = self.size / f32::from(font.units_per_em());

        let mut total = 0.0;

        for letter in self.content.chars() {
            // a letter the font lacks draws as its placeholder box
            let id = font.glyph_index(letter).unwrap_or_default();

            let advance = font
                .glyph_hor_advance(id)
                .expect("failed to read letter advance");

            total += f32::from(advance) * units;
        }

        Size::Fixed(total)
    }

    fn height(&self) -> Size {
        let font = font::load(self.font.as_deref());

        // fonts measure in their own units, this turns them into pixels
        let units = self.size / f32::from(font.units_per_em());

        let ascent = f32::from(font.ascender()) * units;
        let descent = f32::from(font.descender()) * units;

        // descent is negative, so this adds the part below the baseline
        Size::Fixed(ascent - descent)
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let font = font::load(self.font.as_deref());

        renderer.text(&self.content, font, self.size, self.color, area);
    }
}
