mod compositor;
mod connection;
mod layer;
mod output;
mod registry;
mod shm;
mod timer;

use smithay_client_toolkit::{
    compositor::{CompositorState, FrameCallbackData},
    delegate_dispatch2, delegate_registry,
    output::OutputState,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::RegistryState,
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerSurface},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{QueueHandle, globals::registry_queue_init};

use crate::{
    LayerWindow,
    graphics::{Color, Rect, Renderer},
    services::store,
};

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

impl WaylandState {
    fn redraw(&mut self) {
        let (width, height) = (self.width, self.height);

        if width == 0 || height == 0 {
            return;
        }

        // the view runs again on every redraw, so it shows the services as they are now
        let window = (self.view)();

        let Some(root) = window.root else {
            panic!("failed to draw window: no child set");
        };

        // the window is measured in logical pixels, the buffer in real ones
        let buffer_width = width * self.scale as u32;
        let buffer_height = height * self.scale as u32;

        let mut renderer = Renderer::new(buffer_width, buffer_height, self.scale);

        renderer.clear(Color::TRANSPARENT);

        let area = Rect::new(
            0.0,
            0.0,
            root.width().resolve(width as f32),
            root.height().resolve(height as f32),
        );

        root.draw(&mut renderer, area);

        let pixels = renderer.into_argb8888();

        let buffer = shm::create_buffer(&mut self.pool, buffer_width, buffer_height, &pixels);

        let surface = self.layer_surface.wl_surface();

        surface.set_buffer_scale(self.scale as i32);

        surface.damage_buffer(0, 0, buffer_width as i32, buffer_height as i32);

        buffer.attach_to(surface).expect("failed to attach buffer");

        self.layer_surface.commit();
    }

    fn request_frame(&mut self) {
        // changes that land before the next frame all draw together in it
        if self.frame_requested {
            return;
        }

        self.frame_requested = true;

        let surface = self.layer_surface.wl_surface();

        surface.frame(&self.qh, FrameCallbackData(surface.clone()));

        self.layer_surface.commit();
    }
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
