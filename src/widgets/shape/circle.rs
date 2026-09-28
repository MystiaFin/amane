use crate::Size;
use crate::graphics::{self, PathBuilder};

use super::{Shape, Style, round};

pub struct Circle {
    pub(crate) center: Option<(f32, f32)>,
    pub(crate) radius: Size,
    pub(crate) style: Style,
}

impl Circle {
    // sits in the middle of the canvas, as big as fits
    pub fn new() -> Self {
        Self {
            center: None,
            radius: Size::Parent,
            style: Style::default(),
        }
    }

    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center = Some((x, y));

        self
    }

    pub fn radius(mut self, radius: impl Into<Size>) -> Self {
        self.radius = radius.into();

        self
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Circle {
    fn trace(&self, width: f32, height: f32) -> Option<graphics::Path> {
        let (center_x, center_y, radius) =
            round::place(self.center, self.radius, &self.style, width, height);

        let mut path = PathBuilder::new();

        path.arc(center_x, center_y, radius, 0.0, 360.0);
        path.close();

        path.finish()
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
