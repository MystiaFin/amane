use crate::{Horizontal, Keyboard, Layer, LayerWindow, Margin, Vertical, WindowSize, Zone};

// the parts of the window the compositor knows about, kept to see what the next view changes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub width: WindowSize,
    pub height: WindowSize,

    pub vertical: Vertical,
    pub horizontal: Horizontal,

    pub margin: Margin,
    pub layer: Layer,
    pub keyboard: Keyboard,
    pub zone: Zone,

    pub visible: bool,
}

impl From<&LayerWindow> for Settings {
    fn from(window: &LayerWindow) -> Self {
        Self {
            width: window.width,
            height: window.height,

            vertical: window.vertical,
            horizontal: window.horizontal,

            margin: window.margin,
            layer: window.layer,
            keyboard: window.keyboard,
            zone: window.zone,

            visible: window.visible,
        }
    }
}
