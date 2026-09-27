use crate::{LayerWindow, graphics::font, ipc::Handlers, wayland::WaylandApp};

#[derive(Default)]
pub struct App {
    font: Option<String>,
    window: Option<fn() -> LayerWindow>,

    handlers: Handlers,
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

    // lets `amane ipc call <name> [arguments...]` run the handler while the shell is running
    pub fn ipc(mut self, name: &str, handler: fn(&[String]) -> String) -> Self {
        self.handlers.insert(name, handler);

        self
    }

    pub fn run(self) {
        let Some(view) = self.window else {
            panic!("failed to run: no window set");
        };

        if let Some(family) = &self.font {
            font::set_default(family);
        }

        let mut backend = WaylandApp::new(view, self.handlers);

        backend.run();
    }
}
