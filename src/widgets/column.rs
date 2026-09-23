use crate::graphics::Renderer;

use super::{Direction, Layout, Widget};

pub struct Column {
    layout: Layout,
}

impl Column {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            layout: Layout::new(Direction::Column, children),
        }
    }
}

impl Widget for Column {
    fn width(&self) -> f32 {
        self.layout.width()
    }

    fn height(&self) -> f32 {
        self.layout.height()
    }

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32) {
        self.layout.draw(renderer, x, y);
    }
}
