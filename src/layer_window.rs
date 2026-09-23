use crate::Widget;

pub struct LayerWindow {
    pub width: u32,
    pub height: u32,

    pub(crate) root: Box<dyn Widget>,
}

impl LayerWindow {
    pub fn new(width: u32, height: u32, root: impl Widget + 'static) -> Self {
        Self {
            width,
            height,
            root: Box::new(root),
        }
    }
}
