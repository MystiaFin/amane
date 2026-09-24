use crate::{LayerWindow, graphics::font, wayland::WaylandApp};

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

    pub fn font(self, family: &str) -> Self {
        font::set_default(family);

        self
    }
}
