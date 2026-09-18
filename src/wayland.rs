pub mod connection;
pub mod registry;
pub mod surface;

use wayland_client::{
    Connection, EventQueue, delegate_noop,
    protocol::{wl_compositor, wl_registry, wl_surface},
};

pub struct WaylandState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub surface: Option<wl_surface::WlSurface>,
}

impl WaylandState {
    fn new() -> Self {
        Self {
            compositor: None,
            surface: None,
        }
    }
}

pub struct WaylandApp {
    _connection: Connection,
    event_queue: EventQueue<WaylandState>,
    state: WaylandState,
    _registry: wl_registry::WlRegistry,
}

impl WaylandApp {
    pub fn new() -> Self {
        let connection = connection::connect();

        let mut event_queue = connection.new_event_queue();

        let qh = event_queue.handle();

        let mut state = WaylandState::new();

        let registry = registry::request(&connection, &qh);

        event_queue
            .roundtrip(&mut state)
            .expect("failed to initialize Wayland");

        Self {
            _connection: connection,
            event_queue,
            state,
            _registry: registry,
        }
    }

    pub fn has_surface(&self) -> bool {
        self.state.surface.is_some()
    }
}

delegate_noop!(
    WaylandState:
    ignore wl_compositor::WlCompositor
);

delegate_noop!(
    WaylandState:
    ignore wl_surface::WlSurface
);
