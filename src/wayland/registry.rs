use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_compositor, wl_registry},
};

use super::{WaylandState, surface};

pub fn request(connection: &Connection, qh: &QueueHandle<WaylandState>) -> wl_registry::WlRegistry {
    let display = connection.display();

    display.get_registry(qh, ())
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
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
            version,
        } = event
        else {
            return;
        };

        println!("{interface} v{version}");

        if interface == "wl_compositor" {
            let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());

            let surface = surface::create(&compositor, qh);

            state.compositor = Some(compositor);

            state.surface = Some(surface);

            println!("Created wl_surface!");
        }
    }
}
