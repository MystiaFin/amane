use crate::Size;

use super::NeedsHeight;

pub struct NeedsWidth;

impl NeedsWidth {
    pub fn width(self, width: impl Into<Size>) -> NeedsHeight {
        NeedsHeight {
            width: width.into(),
        }
    }
}
