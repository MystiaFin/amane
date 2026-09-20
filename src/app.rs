use crate::{ui::Widget, wayland::WaylandApp};

pub struct App {
    backend: WaylandApp,
}

impl App {
    pub fn new(root: impl Widget + 'static) -> Self {
        Self {
            backend: WaylandApp::new(root),
        }
    }

    pub fn run(&mut self) {
        self.backend.run();
    }
}
