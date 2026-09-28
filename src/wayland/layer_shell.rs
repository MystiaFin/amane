use smithay_client_toolkit::shell::wlr_layer::{
    LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use wayland_client::{Connection, QueueHandle};

use super::{WaylandState, layer};

impl LayerShellHandler for WaylandState {
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let (width, height) = configure.new_size;

        let width = if width == 0 {
            layer::pixels(self.settings.width)
        } else {
            width
        };
        let height = if height == 0 {
            layer::pixels(self.settings.height)
        } else {
            height
        };

        self.width = width;
        self.height = height;

        self.redraw();
    }

    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
}
