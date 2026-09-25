use crate::{Horizontal, Keyboard, Layer, LayerWindow, Margin, Vertical, WindowSize, Zone};

pub struct NeedsHeight {
    pub(crate) width: WindowSize,
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<WindowSize>) -> LayerWindow {
        LayerWindow {
            width: self.width,
            height: height.into(),

            vertical: Vertical::default(),
            horizontal: Horizontal::default(),

            margin: Margin::default(),
            layer: Layer::default(),
            keyboard: Keyboard::default(),
            zone: Zone::default(),

            root: None,
        }
    }
}
