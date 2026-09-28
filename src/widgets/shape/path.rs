use crate::graphics::{self, PathBuilder};

use super::{Shape, Style};

// a free-form outline drawn step by step in canvas coordinates
pub struct Path {
    pub(crate) steps: PathBuilder,
    pub(crate) style: Style,
}

impl Path {
    pub fn new() -> Self {
        Self {
            steps: PathBuilder::new(),
            style: Style::default(),
        }
    }

    // lifts the pen and puts it down here, without drawing
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.steps.move_to(x, y);

        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.steps.line_to(x, y);

        self
    }

    // a curve bent toward the handle point
    pub fn quad_to(mut self, handle_x: f32, handle_y: f32, x: f32, y: f32) -> Self {
        self.steps.quad_to(handle_x, handle_y, x, y);

        self
    }

    // a curve bent by two handle points, the first near the start and the second near the end
    pub fn cubic_to(
        mut self,
        first_x: f32,
        first_y: f32,
        second_x: f32,
        second_y: f32,
        x: f32,
        y: f32,
    ) -> Self {
        self.steps
            .cubic_to(first_x, first_y, second_x, second_y, x, y);

        self
    }

    // same angles as Arc: degrees, 0 straight up, growing clockwise
    pub fn arc(
        mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        start: f32,
        sweep: f32,
    ) -> Self {
        self.steps.arc(center_x, center_y, radius, start, sweep);

        self
    }

    // a straight line back to where this piece started
    pub fn close(mut self) -> Self {
        self.steps.close();

        self
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Path {
    fn trace(&self, _: f32, _: f32) -> Option<graphics::Path> {
        self.steps.clone().finish()
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
