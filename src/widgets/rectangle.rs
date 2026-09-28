mod child;
mod draw;
mod input;
mod needs_height;
mod needs_width;
mod shadow;

use crate::graphics::Color;
use crate::input::Handlers;
use crate::{Fill, Radius, Shadow, Size};

use super::Widget;

pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) fill: Fill,
    pub(crate) radius: Radius,
    pub(crate) border_thickness: f32,
    pub(crate) border_color: Color,
    pub(crate) blur: f32,
    pub(crate) opacity: f32,
    pub(crate) shadow: Option<Shadow>,
    pub(crate) child: Option<Box<dyn Widget>>,
    pub(crate) handlers: Handlers,
}

impl Rectangle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    pub fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();

        self
    }

    pub fn radius(mut self, radius: impl Into<Radius>) -> Self {
        self.radius = radius.into();

        self
    }

    pub fn border(mut self, thickness: f32, color: impl Into<Color>) -> Self {
        self.border_thickness = thickness;
        self.border_color = color.into();

        self
    }

    pub fn blur(mut self, amount: f32) -> Self {
        self.blur = amount;

        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;

        self
    }

    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));

        self
    }
}
