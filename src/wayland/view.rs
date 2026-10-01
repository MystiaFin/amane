use crate::input::KeyHandler;
use crate::{LayerWindow, Monitor, Widget, Window};

// what a window runs on every redraw to find out what it shows
pub enum View {
    Plain(fn() -> LayerWindow),

    // the monitor is replaced when the compositor reports a change to it
    Monitor(fn(&Monitor) -> LayerWindow, Monitor),

    // the name tells it apart from other normal windows
    Normal(&'static str, fn() -> Window),
}

// what a view gave back
pub enum Content {
    Layer(LayerWindow),

    Normal(Window),
}

impl View {
    pub fn run(&self) -> Content {
        match self {
            View::Plain(view) => Content::Layer(view()),
            View::Monitor(view, monitor) => Content::Layer(view(monitor)),
            View::Normal(_, view) => Content::Normal(view()),
        }
    }

    pub fn shows(&self, name: &str) -> bool {
        let View::Normal(shown, _) = self else {
            return false;
        };

        *shown == name
    }
}

impl Content {
    // the name AMANE_FRAMES logs the window under
    pub fn name(&self) -> &'static str {
        match self {
            Content::Layer(window) => window.namespace,
            Content::Normal(_) => "normal",
        }
    }

    // the widgets and the key handler, drawn and used the same way by every kind of window
    pub fn into_parts(self) -> (Option<Box<dyn Widget>>, Option<KeyHandler>) {
        match self {
            Content::Layer(window) => (window.root, window.on_key),
            Content::Normal(window) => (window.root, window.on_key),
        }
    }
}
