use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_surface::WlSurface};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

use super::WaylandState;

pub fn create(
    layer_shell: &ZwlrLayerShellV1,
    surface: &WlSurface,
    qh: &QueueHandle<WaylandState>,
    width: u32,
    height: u32,
) -> ZwlrLayerSurfaceV1 {
    let layer_surface = layer_shell.get_layer_surface(
        surface,
        // Let the compositor choose the monitor.
        None,
        Layer::Overlay,
        "amane".into(),
        qh,
        (),
    );

    layer_surface.set_size(width, height);

    surface.commit();

    layer_surface
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                layer_surface.ack_configure(serial);

                let width = if width == 0 {
                    state.requested_width
                } else {
                    width
                };
                let height = if height == 0 {
                    state.requested_height
                } else {
                    height
                };

                state.width = width;
                state.height = height;

                state.redraw(qh);

                println!("Amane layer window: {width}x{height}");
            }

            zwlr_layer_surface_v1::Event::Closed => {
                layer_surface.destroy();

                state.running = false;
            }

            _ => {}
        }
    }
}
