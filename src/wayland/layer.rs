use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_surface::WlSurface};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

use super::{WaylandState, shm};

use crate::graphics::{Color, Renderer};

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

                if state.buffer.is_some() {
                    return;
                }

                /*
                 * The compositor is allowed to send 0
                 * for width or height.
                 */
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

                let shm = state.shm.as_ref().unwrap();

                let surface = state.surface.as_ref().unwrap();

                /*
                 * Create a pixel canvas using
                 * the actual resolved surface size.
                 */
                let mut renderer = Renderer::new(width, height);

                /*
                 * Start with a transparent background.
                 */
                renderer.clear(Color::TRANSPARENT);

                state.root.draw(&mut renderer, 0.0, 0.0);
                let pixels = renderer.into_argb8888();

                let buffer = shm::create_buffer(shm, qh, width, height, &pixels);
                surface.attach(Some(&buffer), 0, 0);

                surface.damage(0, 0, width as i32, height as i32);

                surface.commit();

                state.buffer = Some(buffer);

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
