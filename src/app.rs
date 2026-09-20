use crate::{LayerWindow, ui::Widget, wayland::WaylandApp};

pub struct App {
    backend: WaylandApp,
}

impl App {
    pub fn new<W>(window: LayerWindow<W>) -> Self
    where
        W: Widget + 'static,
    {
        Self {
            backend: WaylandApp::new(window),
        }
    }

    pub fn run(&mut self) {
        self.backend.run();
    }
}
