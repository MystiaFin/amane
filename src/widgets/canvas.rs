mod draw;
mod needs_height;
mod needs_width;

use crate::Size;

use super::Shape;

pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;

// draws its shapes in its own coordinates, with 0,0 at its top left corner
pub struct Canvas {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) shapes: Vec<Box<dyn Shape>>,
}

impl Canvas {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    // later shapes draw over earlier ones
    pub fn shapes(mut self, shapes: Vec<Box<dyn Shape>>) -> Self {
        self.shapes = shapes;

        self
    }
}
