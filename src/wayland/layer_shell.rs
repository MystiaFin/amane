use smithay_client_toolkit::shell::{
    WaylandSurface,
    wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
};
use wayland_client::{Connection, QueueHandle};

use super::{WaylandState, layer};

impl LayerShellHandler for WaylandState {
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer_surface: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let Some(window) = self.window(layer_surface.wl_surface()) else {
            return;
        };

        let (width, height) = configure.new_size;

        let width = if width == 0 {
            layer::pixels(window.settings.width)
        } else {
            width
        };
        let height = if height == 0 {
            layer::pixels(window.settings.height)
        } else {
            height
        };

        window.width = width;
        window.height = height;

        window.redraw();
    }

    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, layer_surface: &LayerSurface) {
        self.close(layer_surface.wl_surface());
    }
}
