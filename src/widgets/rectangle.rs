mod needs_height;
mod needs_width;

use crate::graphics::{Color, Rect, Renderer};
use crate::{Radius, Size};

use super::Widget;

pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) color: Color,
    pub(crate) radius: Radius,
    pub(crate) border_thickness: f32,
    pub(crate) border_color: Color,
    pub(crate) child: Option<Box<dyn Widget>>,
}

impl Rectangle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn radius(mut self, radius: impl Into<Radius>) -> Self {
        self.radius = radius.into();

        self
    }

    pub fn border(mut self, thickness: f32, color: Color) -> Self {
        self.border_thickness = thickness;
        self.border_color = color;

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));

        self
    }
}

impl Widget for Rectangle {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let radius = self.radius.resolve(area.width, area.height);

        renderer.rectangle(
            area,
            self.color,
            radius,
            self.border_thickness,
            self.border_color,
        );

        let Some(child) = &self.child else {
            return;
        };

        let child_area = Rect::new(
            area.x,
            area.y,
            child.width().resolve(area.width),
            child.height().resolve(area.height),
        );

        child.draw(renderer, child_area);
    }
}
