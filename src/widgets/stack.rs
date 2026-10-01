use crate::graphics::{Rect, Renderer};
use crate::input::Target;
use crate::{Size, Widget};

use super::Direction;
use super::layout::measure;

/*
 * children drawn over each other, the last one on top, all from the
 * stack's top-left corner; move one with Rectangle's translate
 */
pub struct Stack {
    children: Vec<Box<dyn Widget>>,

    // none means as big as the biggest child
    width: Option<Size>,
    height: Option<Size>,
}

impl Stack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            children,
            width: None,
            height: None,
        }
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = Some(width.into());

        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.height = Some(height.into());

        self
    }

    fn child_area(child: &dyn Widget, area: Rect) -> Rect {
        let width = child.width().resolve(area.width);
        let height = child.height().resolve(area.height);

        Rect::new(area.x, area.y, width, height)
    }
}

impl Widget for Stack {
    // a column is as wide as its widest child, which is what a stack needs too
    fn width(&self) -> Size {
        let widest = measure::width(Direction::Column, &self.children, 0.0);

        self.width.unwrap_or(widest)
    }

    // and a row as tall as its tallest
    fn height(&self) -> Size {
        let tallest = measure::height(Direction::Row, &self.children, 0.0);

        self.height.unwrap_or(tallest)
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        for child in &self.children {
            child.draw(renderer, Self::child_area(child.as_ref(), area));
        }
    }

    // later children add their targets later, so the one on top wins
    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        for child in &self.children {
            child.collect_targets(Self::child_area(child.as_ref(), area), targets);
        }
    }
}
