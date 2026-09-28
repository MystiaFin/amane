use crate::{LayerWindow, Monitor, Window};

use super::normal;

// what a window runs on every redraw to find out what it shows
pub enum View {
    Plain(fn() -> LayerWindow),

    // the monitor is replaced when the compositor reports a change to it
    Monitor(fn(&Monitor) -> LayerWindow, Monitor),

    Normal(fn() -> Window),
}

impl View {
    pub fn run(&self) -> LayerWindow {
        match self {
            View::Plain(view) => view(),
            View::Monitor(view, monitor) => view(monitor),
            View::Normal(view) => normal::content(view()),
        }
    }
}
