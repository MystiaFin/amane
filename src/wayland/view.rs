use crate::{LayerWindow, Monitor};

// what a window runs on every redraw to find out what it shows
pub enum View {
    Plain(fn() -> LayerWindow),

    // the monitor is replaced when the compositor reports a change to it
    Monitor(fn(&Monitor) -> LayerWindow, Monitor),
}

impl View {
    pub fn run(&self) -> LayerWindow {
        match self {
            View::Plain(view) => view(),
            View::Monitor(view, monitor) => view(monitor),
        }
    }
}
