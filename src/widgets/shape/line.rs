use crate::graphics::{self, PathBuilder};

use super::{Shape, Style};

// a straight line between two points in canvas coordinates
pub struct Line {
    pub(crate) start: (f32, f32),
    pub(crate) end: (f32, f32),
    pub(crate) style: Style,
}

impl Line {
    pub fn new() -> Self {
        Self {
            start: (0.0, 0.0),
            end: (0.0, 0.0),
            style: Style::default(),
        }
    }

    pub fn from(mut self, x: f32, y: f32) -> Self {
        self.start = (x, y);

        self
    }

    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.end = (x, y);

        self
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Line {
    fn trace(&self, _: f32, _: f32) -> Option<graphics::Path> {
        let (start_x, start_y) = self.start;
        let (end_x, end_y) = self.end;

        let mut path = PathBuilder::new();

        path.move_to(start_x, start_y);
        path.line_to(end_x, end_y);

        path.finish()
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
