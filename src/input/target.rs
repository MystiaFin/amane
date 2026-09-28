use crate::graphics::Rect;

use super::Handlers;

// where a widget was drawn, and what it does when the pointer uses that spot
pub struct Target {
    pub area: Rect,
    pub handlers: Handlers,
}
