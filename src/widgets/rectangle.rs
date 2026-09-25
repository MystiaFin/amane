mod needs_height;
mod needs_width;

use crate::Size;
use crate::graphics::{Color, Rect, Renderer};

use super::Widget;

pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) color: Color,
    pub(crate) radius: f32,
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

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;

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
        renderer.rectangle(area, self.color, self.radius);

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
