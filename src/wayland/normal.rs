use std::mem;
use std::num::NonZeroU32;
use std::ptr;

use smithay_client_toolkit::shell::{
    WaylandSurface,
    xdg::window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
};
use wayland_client::{Connection, QueueHandle};

use crate::window::{CLOSING, REQUESTED};
use crate::{LayerWindow, Window};

use super::{WaylandState, role::Role, settings::Settings, view::View};

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

    // the windows open_window asked for, skipping any already open
    pub fn open_requested(&mut self) {
        let mut queue = REQUESTED.lock().expect("failed to lock requested windows");

        let requested = mem::take(&mut *queue);

        drop(queue);

        for view in requested {
            let mut open = false;

            for window in &self.windows {
                if let View::Normal(shown) = window.view {
                    open = open || ptr::fn_addr_eq(shown, view);
                }
            }

            if open {
                continue;
            }

            self.open_normal(view);
        }
    }

    // the windows close_window asked to close
    pub fn close_requested(&mut self) {
        let mut queue = CLOSING.lock().expect("failed to lock closing windows");

        let closing = mem::take(&mut *queue);

        drop(queue);

        for view in closing {
            let mut surfaces = Vec::new();

            for window in &self.windows {
                if let View::Normal(shown) = window.view {
                    if ptr::fn_addr_eq(shown, view) {
                        surfaces.push(window.role.wl_surface().clone());
                    }
                }
            }

            for surface in surfaces {
                self.close(&surface);
            }
        }
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

        // none means the compositor leaves the size to the window, like 0 does for layers
        let (width, height) = configure.new_size;

        let width = width.map_or(0, NonZeroU32::get);
        let height = height.map_or(0, NonZeroU32::get);

        window.resize(width, height);
    }
}
