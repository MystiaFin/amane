mod button;
mod compositor;
mod connection;
mod frame;
mod key;
mod keyboard;
mod layer;
mod layer_shell;
mod monitor;
mod open;
mod output;
mod pointer;
mod registry;
mod scroll;
mod seat;
mod settings;
mod socket;
mod update;
mod view;
mod wake;
mod window;
mod windows;

use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerShell,
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard::WlKeyboard, wl_pointer::WlPointer, wl_surface::WlSurface},
};

use crate::{LayerWindow, Monitor, ipc::Handlers};

use view::View;
use window::Window;

struct WaylandState {
    // every window has its own layer surface and gpu, and is dropped before the connection
    windows: Vec<Window>,

    // each of these gets a window on every monitor, including ones plugged in later
    per_monitor: Vec<fn(&Monitor) -> LayerWindow>,

    // keys do not say which window they are for, so the window that has focus is kept
    keyboard_focus: Option<WlSurface>,

    registry: RegistryState,
    output: OutputState,
    seat: SeatState,

    // kept so new windows can be made while the shell runs
    compositor: CompositorState,
    layer_shell: LayerShell,
    connection: Connection,

    // kept so they can be released when the mouse or keyboard is unplugged
    pointer_device: Option<WlPointer>,
    keyboard_device: Option<WlKeyboard>,

    qh: QueueHandle<WaylandState>,

    running: bool,
}

// the state is dropped before the event loop, which holds the connection the gpu draws through
pub struct WaylandApp {
    state: WaylandState,

    event_loop: EventLoop<'static, WaylandState>,
}

impl WaylandApp {
    pub fn new(
        views: Vec<fn() -> LayerWindow>,
        per_monitor: Vec<fn(&Monitor) -> LayerWindow>,
        handlers: Handlers,
    ) -> Self {
        let connection = connection::connect();

        let (globals, event_queue) =
            registry_queue_init(&connection).expect("failed to discover Wayland globals");

        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .expect("compositor does not provide wl_compositor");

        let layer_shell =
            LayerShell::bind(&globals, &qh).expect("compositor does not support wlr-layer-shell");

        // windows per monitor are opened once the compositor describes each monitor
        let mut state = WaylandState {
            windows: Vec::new(),

            per_monitor,

            keyboard_focus: None,

            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &qh),
            seat: SeatState::new(&globals, &qh),

            compositor,
            layer_shell,
            connection: connection.clone(),

            pointer_device: None,
            keyboard_device: None,

            qh,

            running: true,
        };

        for view in views {
            state.open(View::Plain(view), None);
        }

        let event_loop = EventLoop::try_new().expect("failed to create event loop");

        WaylandSource::new(connection, event_queue)
            .insert(event_loop.handle())
            .expect("failed to insert Wayland source");

        socket::insert(&event_loop.handle(), handlers);

        wake::insert(&event_loop.handle());

        Self { state, event_loop }
    }

    pub fn run(&mut self) {
        while self.state.running {
            self.event_loop
                .dispatch(None, &mut self.state)
                .expect("failed to dispatch events");
        }
    }
}

delegate_registry!(WaylandState);

delegate_dispatch2!(WaylandState);
