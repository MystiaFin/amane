mod compositor;
mod connection;
mod layer;
mod output;
mod registry;
mod shm;

use smithay_client_toolkit::{
    compositor::CompositorState,
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    registry::RegistryState,
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerSurface},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{Connection, EventQueue, globals::registry_queue_init};

use crate::{
    LayerWindow, Widget,
    graphics::{Color, Rect, Renderer},
};

struct WaylandState {
    root: Box<dyn Widget>,

    requested_width: u32,
    requested_height: u32,

    width: u32,
    height: u32,

    registry: RegistryState,
    output: OutputState,
    shm: Shm,

    pool: SlotPool,

    layer_surface: LayerSurface,

    running: bool,
}

impl WaylandState {
    fn redraw(&mut self) {
        let (width, height) = (self.width, self.height);

        if width == 0 || height == 0 {
            return;
        }

        let mut renderer = Renderer::new(width, height);

        renderer.clear(Color::TRANSPARENT);

        let area = Rect::new(
            0.0,
            0.0,
            self.root.width().resolve(width as f32),
            self.root.height().resolve(height as f32),
        );

        self.root.draw(&mut renderer, area);

        let pixels = renderer.into_argb8888();

        let buffer = shm::create_buffer(&mut self.pool, width, height, &pixels);

        let surface = self.layer_surface.wl_surface();

        surface.damage_buffer(0, 0, width as i32, height as i32);

        buffer.attach_to(surface).expect("failed to attach buffer");

        self.layer_surface.commit();
    }
}

pub struct WaylandApp {
    _connection: Connection,

    event_queue: EventQueue<WaylandState>,

    state: WaylandState,
}

impl WaylandApp {
    pub fn new(window: LayerWindow) -> Self {
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

        let layer_surface = layer::create(&layer_shell, surface, &qh, &window);

        let width = layer::pixels(window.width);
        let height = layer::pixels(window.height);

        let Some(root) = window.root else {
            panic!("failed to create window: no child set");
        };

        // the pool grows on its own once the real size is known
        let pool_size = (width * height * 4).max(4) as usize;

        let pool = SlotPool::new(pool_size, &shm).expect("failed to create shm pool");

        let state = WaylandState {
            root,

            requested_width: width,
            requested_height: height,

            width: 0,
            height: 0,

            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &qh),
            shm,

            pool,

            layer_surface,

            running: true,
        };

        Self {
            _connection: connection,
            event_queue,
            state,
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

delegate_registry!(WaylandState);

delegate_dispatch2!(WaylandState);
