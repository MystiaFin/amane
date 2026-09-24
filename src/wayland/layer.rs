use smithay_client_toolkit::shell::{
    WaylandSurface,
    wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
};
use wayland_client::{Connection, QueueHandle, protocol::wl_surface::WlSurface};

use super::WaylandState;

pub fn create(
    layer_shell: &LayerShell,
    surface: WlSurface,
    qh: &QueueHandle<WaylandState>,
    width: u32,
    height: u32,
) -> LayerSurface {
    let layer_surface =
        layer_shell.create_layer_surface(qh, surface, Layer::Overlay, Some("amane"), None);

    layer_surface.set_size(width, height);

    layer_surface.commit();

    layer_surface
}

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
            self.requested_width
        } else {
            width
        };
        let height = if height == 0 {
            self.requested_height
        } else {
            height
        };

        self.width = width;
        self.height = height;

        self.redraw();

        println!("Amane layer window: {width}x{height}");
    }

    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
}
