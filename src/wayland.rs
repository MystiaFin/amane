mod compositor;
mod connection;
mod frame;
mod layer;
mod output;
mod registry;
mod timer;

use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::RegistryState,
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerSurface},
    },
};
use wayland_client::{Proxy, QueueHandle, globals::registry_queue_init};

use crate::{LayerWindow, graphics::Gpu, services::store};

struct WaylandState {
    view: fn() -> LayerWindow,

    requested_width: u32,
    requested_height: u32,

    width: u32,
    height: u32,

    scale: f32,

    frame_requested: bool,

    registry: RegistryState,
    output: OutputState,

    // the gpu draws into the layer surface, so it has to go first when both are dropped
    gpu: Gpu,

    layer_surface: LayerSurface,

    qh: QueueHandle<WaylandState>,

    running: bool,
}

// the state is dropped before the event loop, which holds the connection the gpu draws through
pub struct WaylandApp {
    state: WaylandState,

    event_loop: EventLoop<'static, WaylandState>,
}

impl WaylandApp {
    pub fn new(view: fn() -> LayerWindow) -> Self {
        let connection = connection::connect();

        let (globals, event_queue) =
            registry_queue_init(&connection).expect("failed to discover Wayland globals");

        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .expect("compositor does not provide wl_compositor");

        let layer_shell =
            LayerShell::bind(&globals, &qh).expect("compositor does not support wlr-layer-shell");

        let surface = compositor.create_surface(&qh);

        // the window's settings are read once here, only its child changes later
        let window = view();

        let layer_surface = layer::create(&layer_shell, surface, &qh, &window);

        let width = layer::pixels(window.width);
        let height = layer::pixels(window.height);

        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = connection.backend().display_ptr().cast();
        let surface = layer_surface.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let state = WaylandState {
            view,

            requested_width: width,
            requested_height: height,

            width: 0,
            height: 0,

            scale: 1.0,

            frame_requested: false,

            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &qh),

            gpu,

            layer_surface,

            qh,

            running: true,
        };

        let event_loop = EventLoop::try_new().expect("failed to create event loop");

        WaylandSource::new(connection, event_queue)
            .insert(event_loop.handle())
            .expect("failed to insert Wayland source");

        Self { state, event_loop }
    }

    pub fn run(&mut self) {
        while self.state.running {
            self.event_loop
                .dispatch(None, &mut self.state)
                .expect("failed to dispatch events");

            // services read for the first time during that dispatch start ticking now
            for ticker in store::take_started() {
                timer::insert(&self.event_loop.handle(), ticker);
            }
        }
    }
}

delegate_registry!(WaylandState);

delegate_dispatch2!(WaylandState);
