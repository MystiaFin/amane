use crate::Size;
use crate::graphics::{self, PathBuilder};

use super::{Shape, Style, round};

/*
 * part of a circle's edge, angles are in degrees,
 * 0 is straight up and they grow clockwise
 */
pub struct Arc {
    pub(crate) center: Option<(f32, f32)>,
    pub(crate) radius: Size,
    pub(crate) start: f32,
    pub(crate) sweep: f32,
    pub(crate) style: Style,
}

impl Arc {
    // a full turn around the middle of the canvas, as big as fits
    pub fn new() -> Self {
        Self {
            center: None,
            radius: Size::Parent,
            start: 0.0,
            sweep: 360.0,
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

    pub fn start(mut self, angle: f32) -> Self {
        self.start = angle;

        self
    }

    // how far the arc goes from its start, a negative sweep goes counterclockwise
    pub fn sweep(mut self, angle: f32) -> Self {
        self.sweep = angle;

        self
    }
}

impl Default for Arc {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Arc {
    fn trace(&self, width: f32, height: f32) -> Option<graphics::BezierPath> {
        // an empty sweep would still draw the line's end caps as a dot
        if self.sweep == 0.0 {
            return None;
        }

        let (center_x, center_y, radius) =
            round::place(self.center, self.radius, &self.style, width, height);

        let mut path = PathBuilder::new();

        path.arc(center_x, center_y, radius, self.start, self.sweep);

        path.finish()
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
