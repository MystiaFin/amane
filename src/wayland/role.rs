use smithay_client_toolkit::{
    session_lock::SessionLockSurface,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
};
use wayland_client::protocol::wl_surface::WlSurface;

// what a window is to the compositor: a layer like a bar, or one screen of the session lock
pub enum Role {
    Layer(LayerSurface),

    Lock(SessionLockSurface),
}

impl Role {
    pub fn wl_surface(&self) -> &WlSurface {
        match self {
            Role::Layer(layer_surface) => layer_surface.wl_surface(),
            Role::Lock(lock_surface) => lock_surface.wl_surface(),
        }
    }

    pub fn commit(&self) {
        self.wl_surface().commit();
    }
}
