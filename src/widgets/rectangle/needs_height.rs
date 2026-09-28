use crate::input::Handlers;
use crate::{Align, Color, Fill, Padding, Radius, Rectangle, Size};

pub struct NeedsHeight {
    pub(crate) width: Size,
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Rectangle {
        Rectangle {
            width: self.width,
            height: height.into(),
            radius: Radius::Fixed(0.0),
            fill: Fill::Color(Color::TRANSPARENT),
            border_thickness: 0.0,
            border_color: Color::TRANSPARENT,
            blur: 0.0,
            opacity: 1.0,
            shadow: None,
            child: None,
            clip: false,
            rotation: 0.0,
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            padding: Padding::default(),
            child_horizontal: Align::Start,
            child_vertical: Align::Start,
            handlers: Handlers::default(),
        }
    }
}
