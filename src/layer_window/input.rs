use std::rc::Rc;

use crate::{Key, LayerWindow};

impl LayerWindow {
    // keys only arrive while the window has keyboard focus, see Keyboard
    pub fn on_key(mut self, handler: impl Fn(Key) + 'static) -> Self {
        self.on_key = Some(Rc::new(handler));

        self
    }
}
