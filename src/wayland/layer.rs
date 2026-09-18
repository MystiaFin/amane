use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_surface::WlSurface};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

use super::{WaylandState, shm};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 120;

pub fn create(
    layer_shell: &ZwlrLayerShellV1,
    surface: &WlSurface,
    qh: &QueueHandle<WaylandState>,
) -> ZwlrLayerSurfaceV1 {
    let layer_surface = layer_shell.get_layer_surface(
        surface,
        // Let the compositor choose
        // which monitor to use.
        None,
        Layer::Overlay,
        "amane".into(),
        qh,
        (),
    );

    layer_surface.set_size(DEFAULT_WIDTH, DEFAULT_HEIGHT);

    /*
     * IMPORTANT:
     *
     * First commit MUST NOT have
     * a buffer attached.
     *
     * This asks the compositor:
     *
     * "please configure my layer surface."
     */
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
                /*
                 * Tell the compositor:
                 *
                 * "I received this configuration."
                 */
                layer_surface.ack_configure(serial);

                /*
                 * For this first experiment,
                 * draw only once.
                 */
                if state.buffer.is_some() {
                    return;
                }

                let width = if width == 0 { DEFAULT_WIDTH } else { width };

                let height = if height == 0 { DEFAULT_HEIGHT } else { height };

                let shm = state.shm.as_ref().unwrap();

                let surface = state.surface.as_ref().unwrap();

                let buffer = shm::create_solid_buffer(shm, qh, width, height);

                /*
                 * Put our pixels on the surface.
                 */
                surface.attach(Some(&buffer), 0, 0);

                /*
                 * Tell compositor the whole
                 * surface changed.
                 */
                surface.damage(0, 0, width as i32, height as i32);

                /*
                 * Present it.
                 */
                surface.commit();

                state.buffer = Some(buffer);

                println!("Amane rectangle: {width}x{height}");
            }

            zwlr_layer_surface_v1::Event::Closed => {
                layer_surface.destroy();

                state.running = false;
            }

            _ => {}
        }
    }
}
