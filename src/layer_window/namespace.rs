use crate::LayerWindow;

impl LayerWindow {
    // the name compositors match rules on, like niri's layer-rule; only read when the window opens
    pub fn namespace(mut self, namespace: &'static str) -> Self {
        self.namespace = namespace;

        self
    }
}
