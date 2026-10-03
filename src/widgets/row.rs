use crate::graphics::{Area, Renderer};
use crate::input::Target;
use crate::{Align, Justify, Size};

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

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.layout.width = Some(width.into());

        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.layout.height = Some(height.into());

        self
    }

    // empty space between each pair of children
    pub fn gap(mut self, gap: f32) -> Self {
        self.layout.set_gap(gap);

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

impl Widget for Row {
    fn width(&self) -> Size {
        self.layout.width()
    }

    fn height(&self) -> Size {
        self.layout.height()
    }

    fn draw(&self, renderer: &mut Renderer, area: Area) {
        self.layout.draw(renderer, area);
    }

    fn collect_targets(&self, area: Area, targets: &mut Vec<Target>) {
        self.layout.collect_targets(area, targets);
    }
}
