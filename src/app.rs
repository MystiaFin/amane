use crate::{LayerWindow, Monitor, graphics::font, ipc::Handlers, wayland::WaylandApp};

#[derive(Default)]
pub struct App {
    font: Option<String>,

    windows: Vec<fn() -> LayerWindow>,
    per_monitor: Vec<fn(&Monitor) -> LayerWindow>,

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

    // each call adds one more window, on the monitor the compositor chooses
    pub fn window(mut self, view: fn() -> LayerWindow) -> Self {
        self.windows.push(view);

        self
    }

    // one window on every monitor, following monitors as they are plugged in and out
    pub fn window_per_monitor(mut self, view: fn(&Monitor) -> LayerWindow) -> Self {
        self.per_monitor.push(view);

        self
    }

    // lets `amane ipc call <name> [arguments...]` run the handler while the shell is running
    pub fn ipc(mut self, name: &str, handler: fn(&[String]) -> String) -> Self {
        self.handlers.insert(name, handler);

        self
    }

    pub fn run(self) {
        if self.windows.is_empty() && self.per_monitor.is_empty() {
            panic!("failed to run: no window set");
        }

        if let Some(family) = &self.font {
            font::set_default(family);
        }

        let mut backend = WaylandApp::new(self.windows, self.per_monitor, self.handlers);

        backend.run();
    }
}
