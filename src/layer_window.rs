mod horizontal;
mod keyboard;
mod layer;
mod margin;
mod needs_height;
mod needs_width;
mod vertical;
mod window_size;

use crate::Widget;

pub use horizontal::Horizontal;
pub use keyboard::Keyboard;
pub use layer::Layer;
pub use margin::Margin;
pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;
pub use vertical::Vertical;
pub use window_size::WindowSize;

pub struct LayerWindow {
    pub(crate) width: WindowSize,
    pub(crate) height: WindowSize,

    pub(crate) vertical: Vertical,
    pub(crate) horizontal: Horizontal,

    pub(crate) margin: Margin,
    pub(crate) layer: Layer,
    pub(crate) keyboard: Keyboard,

    pub(crate) root: Option<Box<dyn Widget>>,
}

impl LayerWindow {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    pub fn anchor_vertical(mut self, vertical: Vertical) -> Self {
        self.vertical = vertical;

        self
    }

    pub fn anchor_horizontal(mut self, horizontal: Horizontal) -> Self {
        self.horizontal = horizontal;

        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;

        self
    }

    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;

        self
    }

    pub fn keyboard(mut self, keyboard: Keyboard) -> Self {
        self.keyboard = keyboard;

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.root = Some(Box::new(child));

        self
    }
}
