use crate::{Color, Radius, Rectangle, Size};

pub struct NeedsHeight {
    pub(crate) width: Size,
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Rectangle {
        Rectangle {
            width: self.width,
            height: height.into(),
            radius: Radius::Fixed(0.0),
            color: Color::TRANSPARENT,
            border_thickness: 0.0,
            border_color: Color::TRANSPARENT,
            opacity: 1.0,
            child: None,
        }
    }
}
