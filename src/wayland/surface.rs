use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor::WlCompositor, wl_surface::WlSurface},
};

use super::WaylandState;

pub fn create(compositor: &WlCompositor, qh: &QueueHandle<WaylandState>) -> WlSurface {
    compositor.create_surface(qh, ())
}
