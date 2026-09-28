mod button;
mod compositor;
mod connection;
mod frame;
mod key;
mod keyboard;
mod layer;
mod layer_shell;
mod output;
mod pointer;
mod registry;
mod scroll;
mod seat;
mod settings;
mod socket;
mod timer;
mod update;

use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerSurface},
    },
};
use wayland_client::{
    Proxy, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard::WlKeyboard, wl_pointer::WlPointer},
};

use crate::{
    LayerWindow,
    graphics::Gpu,
    input::{KeyHandler, Pointer},
    ipc::Handlers,
    services::store,
};

use settings::Settings;

struct WaylandState {
    view: fn() -> LayerWindow,

    // what the compositor was last told, so only a real change is sent again
    settings: Settings,

    width: u32,
    height: u32,

    scale: f32,

    frame_requested: bool,

    registry: RegistryState,
    output: OutputState,
    seat: SeatState,

    // kept so they can be released when the mouse or keyboard is unplugged
    pointer_device: Option<WlPointer>,
    keyboard_device: Option<WlKeyboard>,

    // both come from the last drawn view, so input matches what is on screen
    pointer: Pointer,
    on_key: Option<KeyHandler>,

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
    pub fn new(view: fn() -> LayerWindow, handlers: Handlers) -> Self {
        let connection = connection::connect();

        let (globals, event_queue) =
            registry_queue_init(&connection).expect("failed to discover Wayland globals");

        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .expect("compositor does not provide wl_compositor");

        let layer_shell =
            LayerShell::bind(&globals, &qh).expect("compositor does not support wlr-layer-shell");

        let surface = compositor.create_surface(&qh);

        // later views can change these, each redraw compares them with the last ones
        let window = view();

        let settings = Settings::from(&window);

        let layer_surface = layer::create(&layer_shell, surface, &qh, &settings);

        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = connection.backend().display_ptr().cast();
        let surface = layer_surface.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let state = WaylandState {
            view,

            settings,

            width: 0,
            height: 0,

            scale: 1.0,

            frame_requested: false,

            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &qh),
            seat: SeatState::new(&globals, &qh),

            pointer_device: None,
            keyboard_device: None,

            pointer: Pointer::default(),
            on_key: None,

            gpu,

            layer_surface,

            qh,

            running: true,
        };

        let event_loop = EventLoop::try_new().expect("failed to create event loop");

        WaylandSource::new(connection, event_queue)
            .insert(event_loop.handle())
            .expect("failed to insert Wayland source");

        socket::insert(&event_loop.handle(), handlers);

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
