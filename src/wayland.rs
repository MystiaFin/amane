mod connection;
mod layer;
mod registry;
mod shm;
mod surface;

use wayland_client::{
    Connection, EventQueue, QueueHandle, delegate_noop,
    protocol::{wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface},
};

use crate::{
    LayerWindow,
    ui::Widget,
};

use crate::graphics::{Color, Renderer};

use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

struct WaylandState {
    root: Box<dyn Widget>,

    requested_width: u32,
    requested_height: u32,

    width: u32,
    height: u32,

    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,

    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,

    surface: Option<wl_surface::WlSurface>,

    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,

    running: bool,
}

impl WaylandState {
    fn new(root: Box<dyn Widget>, requested_width: u32, requested_height: u32) -> Self {
        Self {
            root,
            requested_width,
            requested_height,

            width: 0,
            height: 0,

            compositor: None,
            shm: None,
            layer_shell: None,

            surface: None,
            layer_surface: None,

            running: true,
        }
    }

    fn redraw(&mut self, qh: &QueueHandle<Self>) {
        let (width, height) = (self.width, self.height);
        if width == 0 || height == 0 {
            return;
        }

        let mut renderer = Renderer::new(width, height);
        renderer.clear(Color::TRANSPARENT);
        self.root.draw(&mut renderer, 0.0, 0.0);
        let pixels = renderer.into_argb8888();

        let shm = self.shm.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        let buffer = shm::create_buffer(shm, qh, width, height, &pixels);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage(0, 0, width as i32, height as i32);
        surface.commit();
    }

    fn ensure_required_globals(&self) {
        assert!(
            self.compositor.is_some(),
            "compositor does not provide wl_compositor"
        );

        assert!(self.shm.is_some(), "compositor does not provide wl_shm");

        assert!(
            self.layer_shell.is_some(),
            "compositor does not support wlr-layer-shell"
        );
    }

    fn create_layer_surface(&mut self, qh: &QueueHandle<Self>) {
        let compositor = self.compositor.as_ref().unwrap();

        let layer_shell = self.layer_shell.as_ref().unwrap();

        let surface = surface::create(compositor, qh);

        let layer_surface = layer::create(
            layer_shell,
            &surface,
            qh,
            self.requested_width,
            self.requested_height,
        );

        self.surface = Some(surface);

        self.layer_surface = Some(layer_surface);
    }
}

pub struct WaylandApp {
    _connection: Connection,

    event_queue: EventQueue<WaylandState>,

    state: WaylandState,

    _registry: wl_registry::WlRegistry,
}

impl WaylandApp {
    pub fn new(window: LayerWindow) -> Self {
        let LayerWindow {
            width,
            height,
            root,
        } = window;

        let connection = connection::connect();

        let mut event_queue = connection.new_event_queue();

        let qh = event_queue.handle();

        let mut state = WaylandState::new(root, width, height);

        let registry = registry::request(&connection, &qh);

        event_queue
            .roundtrip(&mut state)
            .expect("failed to discover Wayland globals");

        state.ensure_required_globals();

        state.create_layer_surface(&qh);

        Self {
            _connection: connection,
            event_queue,
            state,
            _registry: registry,
        }
    }

    pub fn run(&mut self) {
        while self.state.running {
            self.event_queue
                .blocking_dispatch(&mut self.state)
                .expect("Wayland event loop failed");
        }
    }
}

/*
 * These objects can generate events,
 * but we don't care about their events yet.
 */

delegate_noop!(
    WaylandState:
    ignore wl_compositor::WlCompositor
);

delegate_noop!(
    WaylandState:
    ignore wl_surface::WlSurface
);

delegate_noop!(
    WaylandState:
    ignore wl_shm::WlShm
);

delegate_noop!(
    WaylandState:
    ignore wl_shm_pool::WlShmPool
);

delegate_noop!(
    WaylandState:
    ignore zwlr_layer_shell_v1::ZwlrLayerShellV1
);
