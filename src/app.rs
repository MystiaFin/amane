use crate::{LayerWindow, graphics::font, wayland::WaylandApp};

#[derive(Default)]
pub struct App {
    font: Option<String>,
    window: Option<fn() -> LayerWindow>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font(mut self, family: &str) -> Self {
        self.font = Some(String::from(family));

        self
    }

    pub fn window(mut self, view: fn() -> LayerWindow) -> Self {
        self.window = Some(view);

        self
    }

    pub fn run(self) {
        let Some(view) = self.window else {
            panic!("failed to run: no window set");
        };

        if let Some(family) = &self.font {
            font::set_default(family);
        }

        let mut backend = WaylandApp::new(view);

        backend.run();
    }
}
