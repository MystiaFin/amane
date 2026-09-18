pub mod connection;
pub mod registry;
pub mod surface;

use wayland_client:: {
    delegate_noop,
    protocol::{
        wl_compositor,
        wl_surface,
    },
};

pub struct WaylandState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub surface: Option<wl_surface::WlSurface>,
}

impl WaylandState {
    pub fn new() -> Self {
        Self {
            compositor: None,
            surface: None,
        }
    }
}

delegate_noop!(WaylandState: ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandState: ignore wl_surface::WlSurface);
