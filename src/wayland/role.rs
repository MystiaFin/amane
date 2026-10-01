use smithay_client_toolkit::{
    session_lock::SessionLockSurface,
    shell::{WaylandSurface, wlr_layer::LayerSurface, xdg::window::Window as XdgWindow},
};
use wayland_client::protocol::wl_surface::WlSurface;

use super::settings::Settings;

// what a window is to the compositor: a layer like a bar, one screen of the session lock, or a normal window
pub enum Role {
    Layer {
        surface: LayerSurface,

        // what the compositor was last told, so only a real change is sent again
        settings: Settings,
    },

    Lock(SessionLockSurface),

    Normal {
        window: XdgWindow,

        // the size it opened at, kept for as long as the compositor leaves the size to it
        size: (u32, u32),
    },
}

impl Role {
    pub fn wl_surface(&self) -> &WlSurface {
        match self {
            Role::Layer { surface, .. } => surface.wl_surface(),
            Role::Lock(lock_surface) => lock_surface.wl_surface(),
            Role::Normal { window, .. } => window.wl_surface(),
        }
    }

    pub fn commit(&self) {
        self.wl_surface().commit();
    }
}
