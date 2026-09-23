use crate::graphics::Renderer;

use super::{Direction, Layout, Widget};

pub struct Row {
    layout: Layout,
}

impl Row {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            layout: Layout::new(Direction::Row, children),
        }
    }
}

impl Widget for Row {
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
