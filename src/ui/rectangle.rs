use crate::graphics::{Color, Rect, Renderer};

use super::Widget;

pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

impl Widget for Rectangle {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32) {
        renderer.rectangle(Rect::new(x, y, self.width, self.height), self.color);
    }
}
