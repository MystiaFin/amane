use crate::{Color, Rectangle, Size};

pub struct NeedsHeight {
    pub(crate) width: Size,
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Rectangle {
        Rectangle {
            width: self.width,
            height: height.into(),
            radius: 0.0,
            color: Color::TRANSPARENT,
            child: None,
        }
    }
}
