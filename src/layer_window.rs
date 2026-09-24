mod horizontal;
mod keyboard;
mod layer;
mod margin;
mod size;
mod vertical;

use crate::Widget;

pub use horizontal::Horizontal;
pub use keyboard::Keyboard;
pub use layer::Layer;
pub use margin::Margin;
pub use size::Size;
pub use vertical::Vertical;

pub struct LayerWindow {
    pub(crate) width: Size,
    pub(crate) height: Size,

    pub(crate) vertical: Vertical,
    pub(crate) horizontal: Horizontal,

    pub(crate) margin: Margin,
    pub(crate) layer: Layer,
    pub(crate) keyboard: Keyboard,

    pub(crate) root: Option<Box<dyn Widget>>,
}

impl LayerWindow {
    pub fn new(width: impl Into<Size>, height: impl Into<Size>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),

            vertical: Vertical::default(),
            horizontal: Horizontal::default(),

            margin: Margin::default(),
            layer: Layer::default(),
            keyboard: Keyboard::default(),

            root: None,
        }
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

    pub fn root(mut self, root: impl Widget + 'static) -> Self {
        self.root = Some(Box::new(root));

        self
    }
}
