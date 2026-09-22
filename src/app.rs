use crate::{LayerWindow, wayland::WaylandApp};

pub struct App {
    backend: WaylandApp,
}

impl App {
    pub fn new(window: LayerWindow) -> Self {
        Self {
            backend: WaylandApp::new(window),
        }
    }

    pub fn run(&mut self) {
        self.backend.run();
    }
}
