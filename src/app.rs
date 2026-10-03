use crate::allocator;
use crate::graphics::font;
use crate::ipc::IpcHandlers;
use crate::wayland::WaylandApp;
use crate::{LayerWindow, Monitor, Window};

#[derive(Default)]
pub struct App {
    font: Option<String>,

    windows: Vec<fn() -> LayerWindow>,
    normal_windows: Vec<(&'static str, fn() -> Window)>,
    per_monitor: Vec<fn(&Monitor) -> LayerWindow>,
    lock: Option<fn(&Monitor) -> LayerWindow>,

    handlers: IpcHandlers,
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

    // a regular desktop window with a title bar, next to layer windows; close_window takes the name
    pub fn normal_window(mut self, name: &'static str, view: fn() -> Window) -> Self {
        self.normal_windows.push((name, view));

        self
    }

    // one window on every monitor, following monitors as they are plugged in and out
    pub fn window_per_monitor(mut self, view: fn(&Monitor) -> LayerWindow) -> Self {
        self.per_monitor.push(view);

        self
    }

    /*
     * the lock screen, shown on every monitor once Lock::start locks the session;
     * it stays locked until Lock::unlock gets a password pam accepts
     */
    pub fn lock(mut self, view: fn(&Monitor) -> LayerWindow) -> Self {
        self.lock = Some(view);

        self
    }

    // lets `amane ipc call <name> [arguments...]` run the handler while the shell is running
    pub fn ipc(mut self, name: &str, handler: fn(&[String]) -> String) -> Self {
        self.handlers.insert(name, handler);

        self
    }

    pub fn run(self) {
        let no_windows = self.windows.is_empty() && self.normal_windows.is_empty();

        if no_windows && self.per_monitor.is_empty() && self.lock.is_none() {
            panic!("failed to run: no window set");
        }

        allocator::limit();

        if let Some(family) = &self.font {
            font::set_default(family);
        }

        let mut backend = WaylandApp::new(
            self.windows,
            self.normal_windows,
            self.per_monitor,
            self.lock,
            self.handlers,
        );

        backend.run();
    }
}
