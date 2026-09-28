use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_region::{self, WlRegion}};

use crate::LayerWindow;

use super::{WaylandState, window::Window};

impl Window {
    // only a changed region is sent, it takes effect with the next commit like the rest
    pub fn update_input_region(&mut self, window: &LayerWindow) {
        if window.input_region == self.input_region {
            return;
        }

        self.input_region = window.input_region.clone();

        let surface = self.layer_surface.wl_surface();

        // no region set means the whole window takes the pointer again
        let Some(areas) = &self.input_region else {
            surface.set_input_region(None);

            return;
        };

        let region = self.compositor.create_region(&self.qh, ());

        for area in areas {
            region.add(area.x, area.y, area.width, area.height);
        }

        surface.set_input_region(Some(&region));

        // the surface keeps its own copy, so the region can go right away
        region.destroy();
    }
}

// a region never sends events, but wayland-client still needs somewhere to send them
impl Dispatch<WlRegion, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
