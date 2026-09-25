use crate::WindowSize;

use super::NeedsHeight;

pub struct NeedsWidth;

impl NeedsWidth {
    pub fn width(self, width: impl Into<WindowSize>) -> NeedsHeight {
        NeedsHeight {
            width: width.into(),
        }
    }
}
