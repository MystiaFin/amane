use wayland_client::protocol::wl_surface::WlSurface;

use super::{WaylandState, window::Window};

impl WaylandState {
    // wayland events name the surface they are about, this finds its window
    pub fn window(&mut self, surface: &WlSurface) -> Option<&mut Window> {
        for window in &mut self.windows {
            if window.role.wl_surface() == surface {
                return Some(window);
            }
        }

        None
    }

    // a service or handler may have changed what any of the windows shows
    pub fn request_frames(&mut self) {
        for window in &mut self.windows {
            window.request_frame();
        }
    }

    pub fn close(&mut self, surface: &WlSurface) {
        self.windows
            .retain(|window| window.role.wl_surface() != surface);

        // windows made per monitor come back when a monitor is plugged in again
        if self.windows.is_empty() && self.per_monitor.is_empty() {
            self.running = false;
        }
    }
}
