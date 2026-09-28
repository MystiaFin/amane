use crate::graphics::{Rect, Renderer};
use crate::input::Target;
use crate::{Align, Justify, Size};

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

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.layout.width = width.into();

        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.layout.height = height.into();

        self
    }

    pub fn justify(mut self, justify: impl Into<Justify>) -> Self {
        self.layout.justify = justify.into();

        self
    }

    pub fn align(mut self, align: impl Into<Align>) -> Self {
        self.layout.align = align.into();

        self
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

    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        self.layout.collect_targets(area, targets);
    }
}
