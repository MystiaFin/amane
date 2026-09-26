mod compositor;
mod connection;
mod frame;
mod layer;
mod output;
mod registry;
mod shm;
mod timer;

use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::RegistryState,
    shell::wlr_layer::{LayerShell, LayerSurface},
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{QueueHandle, globals::registry_queue_init};

use crate::{LayerWindow, services::store};

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
    shm: Shm,

    pool: SlotPool,

    layer_surface: LayerSurface,

    qh: QueueHandle<WaylandState>,

    running: bool,
}

pub struct WaylandApp {
    event_loop: EventLoop<'static, WaylandState>,

    state: WaylandState,
}

impl WaylandApp {
    pub fn new(view: fn() -> LayerWindow) -> Self {
        let connection = connection::connect();

        let (globals, event_queue) =
            registry_queue_init(&connection).expect("failed to discover Wayland globals");

        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .expect("compositor does not provide wl_compositor");

        let shm = Shm::bind(&globals, &qh).expect("compositor does not provide wl_shm");

        let layer_shell =
            LayerShell::bind(&globals, &qh).expect("compositor does not support wlr-layer-shell");

        let surface = compositor.create_surface(&qh);

        // the window's settings are read once here, only its child changes later
        let window = view();

        let layer_surface = layer::create(&layer_shell, surface, &qh, &window);

        let width = layer::pixels(window.width);
        let height = layer::pixels(window.height);

        // the pool grows on its own once the real size is known
        let pool_size = (width * height * 4).max(4) as usize;

        let pool = SlotPool::new(pool_size, &shm).expect("failed to create shm pool");

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
            shm,

            pool,

            layer_surface,

            qh,

            running: true,
        };

        let event_loop = EventLoop::try_new().expect("failed to create event loop");

        WaylandSource::new(connection, event_queue)
            .insert(event_loop.handle())
            .expect("failed to insert Wayland source");

        Self { event_loop, state }
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
