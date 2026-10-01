mod horizontal;
mod input_area;
mod keyboard;
mod layer;
mod margin;
mod needs_height;
mod needs_width;
mod vertical;
mod window_size;
mod zone;

use std::rc::Rc;

use crate::input::KeyHandler;
use crate::{Key, Widget};

pub use horizontal::Horizontal;
pub use input_area::InputArea;
pub use keyboard::Keyboard;
pub use layer::Layer;
pub use margin::Margin;
pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;
pub use vertical::Vertical;
pub use window_size::WindowSize;
pub use zone::Zone;

pub struct LayerWindow {
    pub(crate) width: WindowSize,
    pub(crate) height: WindowSize,

    pub(crate) vertical: Vertical,
    pub(crate) horizontal: Horizontal,

    pub(crate) margin: Margin,
    pub(crate) layer: Layer,
    pub(crate) keyboard: Keyboard,
    pub(crate) zone: Zone,

    pub(crate) namespace: &'static str,

    pub(crate) visible: bool,

    // none means the whole window takes the pointer
    pub(crate) input_region: Option<Vec<InputArea>>,

    pub(crate) root: Option<Box<dyn Widget>>,

    pub(crate) on_key: Option<KeyHandler>,
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

    pub fn space(mut self, zone: Zone) -> Self {
        self.zone = zone;

        self
    }

    // the name compositors match rules on, like niri's layer-rule; only read when the window opens
    pub fn namespace(mut self, namespace: &'static str) -> Self {
        self.namespace = namespace;

        self
    }

    // a hidden window keeps running and shows again once the view says so
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;

        self
    }

    // only these areas take the pointer, everywhere else clicks go to the windows below
    pub fn input_region(mut self, areas: Vec<InputArea>) -> Self {
        self.input_region = Some(areas);

        self
    }

    // no area takes the pointer, so every click goes to the windows below
    pub fn click_through(self) -> Self {
        self.input_region(Vec::new())
    }

    // keys only arrive while the window has keyboard focus, see Keyboard
    pub fn on_key(mut self, handler: impl Fn(Key) + 'static) -> Self {
        self.on_key = Some(Rc::new(handler));

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.root = Some(Box::new(child));

        self
    }
}
