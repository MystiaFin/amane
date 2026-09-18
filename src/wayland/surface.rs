use wayland_client::{
    protocol::{
        wl_compositor::WlCompositor,
        wl_surface::WlSurface,
    },
    QueueHandle,
};

use crate::wayland::WaylandState;

pub fn create(
    compositor: &WlCompositor,
    qh: &QueueHandle<WaylandState>,
) -> WlSurface {
    compositor.create_surface(qh, ())
}
