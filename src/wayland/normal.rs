use smithay_client_toolkit::shell::{
    WaylandSurface,
    xdg::window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
};
use wayland_client::{Connection, QueueHandle};

use crate::{LayerWindow, Window};

use super::{WaylandState, layer, role::Role, settings::Settings, view::View};

impl WaylandState {
    pub fn open_normal(&mut self, view: fn() -> Window) {
        let surface = self.compositor.create_surface(&self.qh);

        let window = view();

        // the compositor draws the title bar when it can, amane has none of its own
        let xdg_window =
            self.xdg_shell
                .create_window(surface, WindowDecorations::RequestServer, &self.qh);

        xdg_window.set_title(window.title.clone());
        xdg_window.set_app_id("amane");

        // the same smallest and biggest size is how a window says it cannot be resized
        if !window.resizable {
            let size = Some((window.width as u32, window.height as u32));

            xdg_window.set_min_size(size);
            xdg_window.set_max_size(size);
        }

        // the compositor answers the first commit with a configure, drawing starts there
        xdg_window.commit();

        let content = content(window);

        let settings = Settings::from(&content);

        self.add(View::Normal(view), None, settings, Role::Normal(xdg_window));
    }
}

/*
 * the widgets and key handler, in the form the rest of the backend draws;
 * the size is only used until the compositor gives one
 */
pub fn content(window: Window) -> LayerWindow {
    let mut content = LayerWindow::new()
        .width(window.width)
        .height(window.height);

    content.root = window.root;
    content.on_key = window.on_key;

    content
}

impl WindowHandler for WaylandState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, xdg_window: &XdgWindow) {
        self.close(xdg_window.wl_surface());
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        xdg_window: &XdgWindow,
        configure: WindowConfigure,
        _: u32,
    ) {
        let Some(window) = self.window(xdg_window.wl_surface()) else {
            return;
        };

        // none means the compositor leaves the size to the window
        let (width, height) = configure.new_size;

        let width = match width {
            Some(width) => width.get(),
            None => layer::pixels(window.settings.width),
        };

        let height = match height {
            Some(height) => height.get(),
            None => layer::pixels(window.settings.height),
        };

        window.width = width;
        window.height = height;

        window.redraw();
    }
}
