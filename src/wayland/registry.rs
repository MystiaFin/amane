use wayland_client::{
    protocol::{
        wl_compositor,
        wl_registry,
        wl_shm,
    },
    Connection,
    Dispatch,
    QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1,
};

use super::WaylandState;

pub fn request(
    connection: &Connection,
    qh: &QueueHandle<WaylandState>,
) -> wl_registry::WlRegistry {
    connection
        .display()
        .get_registry(
            qh,
            (),
        )
}

impl Dispatch<
    wl_registry::WlRegistry,
    (),
> for WaylandState
{
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            ..
        } = event
        else {
            return;
        };

        match interface.as_str() {
            "wl_compositor" => {
                let compositor =
                    registry.bind::<
                        wl_compositor::WlCompositor,
                        _,
                        _
                    >(
                        name,
                        1,
                        qh,
                        (),
                    );

                state.compositor =
                    Some(compositor);
            }

            "wl_shm" => {
                let shm =
                    registry.bind::<
                        wl_shm::WlShm,
                        _,
                        _
                    >(
                        name,
                        1,
                        qh,
                        (),
                    );

                state.shm =
                    Some(shm);
            }

            "zwlr_layer_shell_v1" => {
                let layer_shell =
                    registry.bind::<
                        zwlr_layer_shell_v1::ZwlrLayerShellV1,
                        _,
                        _
                    >(
                        name,
                        1,
                        qh,
                        (),
                    );

                state.layer_shell =
                    Some(layer_shell);
            }

            _ => {}
        }
    }
}
