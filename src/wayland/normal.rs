use std::mem;
use std::num::NonZeroU32;
use std::sync::PoisonError;

use smithay_client_toolkit::shell::{
    WaylandSurface,
    xdg::window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
};
use wayland_client::{Connection, QueueHandle};

use crate::Window;
use crate::window::{CLOSING, REQUESTED};

use super::{
    WaylandState,
    surface::{Role, View},
};

impl WaylandState {
    pub fn open_normal(&mut self, name: &'static str, view: fn() -> Window) {
        let surface = self.compositor.create_surface(&self.qh);

        let window = view();

        // the compositor draws the title bar when it can, amane has none of its own
        let xdg_window =
            self.xdg_shell
                .create_window(surface, WindowDecorations::RequestServer, &self.qh);

        xdg_window.set_title(window.title.clone());
        xdg_window.set_app_id("amane");

        let size = (
            self.scale_factor.pixels(window.width),
            self.scale_factor.pixels(window.height),
        );

        // the same smallest and biggest size is how a window says it cannot be resized
        if !window.resizable {
            xdg_window.set_min_size(Some(size));
            xdg_window.set_max_size(Some(size));
        }

        // the compositor answers the first commit with a configure, drawing starts there
        xdg_window.commit();

        let role = Role::Normal {
            window: xdg_window,
            size,
        };

        self.add(View::Normal(name, view), None, role);
    }

    // the windows open_window asked for, skipping any already open
    pub fn open_requested(&mut self) {
        let mut queue = REQUESTED.lock().unwrap_or_else(PoisonError::into_inner);

        let requested = mem::take(&mut *queue);

        drop(queue);

        for (name, view) in requested {
            let mut open = false;

            for window in &self.windows {
                open = open || window.view.shows(name);
            }

            if open {
                continue;
            }

            self.open_normal(name, view);
        }
    }

    // the windows close_window asked to close
    pub fn close_requested(&mut self) {
        let mut queue = CLOSING.lock().unwrap_or_else(PoisonError::into_inner);

        let closing = mem::take(&mut *queue);

        drop(queue);

        for name in closing {
            let mut surfaces = Vec::new();

            for window in &self.windows {
                if window.view.shows(name) {
                    surfaces.push(window.role.wl_surface().clone());
                }
            }

            for surface in surfaces {
                self.close(&surface);
            }
        }
    }
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
