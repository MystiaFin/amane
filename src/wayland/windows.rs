use wayland_client::protocol::wl_surface::WlSurface;

use crate::services::wake;

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

    // input only changes the window it arrived in, besides the services it wrote
    pub fn request_frame_on(&mut self, surface: &WlSurface) {
        if let Some(window) = self.window(surface) {
            window.request_frame();
        }
    }

    // only windows that read a service that changed draw again
    pub fn request_changed_frames(&mut self) {
        let Some(changed) = wake::take_changes() else {
            self.request_frames();

            return;
        };

        for window in &mut self.windows {
            if window.reads.is_disjoint(&changed) {
                continue;
            }

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
