use std::rc::Rc;

use crate::input::KeyHandler;
use crate::{Key, Widget};

// a regular desktop window with a title bar, like a settings app, next to the layer windows
pub struct Window {
    pub(crate) title: String,

    // the size it opens at, the user can resize it after that if it is resizable
    pub(crate) width: f32,
    pub(crate) height: f32,

    pub(crate) resizable: bool,

    pub(crate) root: Option<Box<dyn Widget>>,

    pub(crate) on_key: Option<KeyHandler>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            title: String::from("amane"),

            width: 640.0,
            height: 480.0,

            resizable: true,

            root: None,

            on_key: None,
        }
    }

    // read once, when the window opens
    pub fn title(mut self, title: &str) -> Self {
        self.title = String::from(title);

        self
    }

    // read once, when the window opens
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;

        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.root = Some(Box::new(child));

        self
    }

    pub fn on_key(mut self, handler: impl Fn(Key) + 'static) -> Self {
        self.on_key = Some(Rc::new(handler));

        self
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
