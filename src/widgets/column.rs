use crate::Size;
use crate::graphics::{Rect, Renderer};

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
    fn width(&self) -> Size {
        self.layout.width()
    }

    fn height(&self) -> Size {
        self.layout.height()
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        self.layout.draw(renderer, area);
    }
}
