use crate::{LayerWindow, graphics::font, wayland::WaylandApp};

#[derive(Default)]
pub struct App {
    font: Option<String>,
    window: Option<LayerWindow>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font(mut self, family: &str) -> Self {
        self.font = Some(String::from(family));

        self
    }

    pub fn window(mut self, window: LayerWindow) -> Self {
        self.window = Some(window);

        self
    }

    pub fn run(self) {
        let Some(window) = self.window else {
            panic!("failed to run: no window set");
        };

        if let Some(family) = &self.font {
            font::set_default(family);
        }

        let mut backend = WaylandApp::new(window);

        backend.run();
    }
}
