use crate::LayerWindow;

impl LayerWindow {
    // a hidden window keeps running and shows again once the view says so
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;

        self
    }
}
