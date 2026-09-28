use crate::{Canvas, Size};

pub struct NeedsHeight {
    pub(crate) width: Size,
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Canvas {
        Canvas {
            width: self.width,
            height: height.into(),
            shapes: Vec::new(),
        }
    }
}
