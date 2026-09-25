use crate::Size;
use crate::graphics::{Rect, Renderer};

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
