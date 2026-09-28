use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{Proxy, protocol::wl_output::WlOutput};

use crate::graphics::Gpu;
use crate::input::Pointer;

use super::{WaylandState, layer, settings::Settings, view::View, window::Window};

impl WaylandState {
    pub fn open(&mut self, view: View, output: Option<WlOutput>) {
        let surface = self.compositor.create_surface(&self.qh);

        // later views can change these, each redraw compares them with the last ones
        let window = view.run();

        let settings = Settings::from(&window);

        let layer_surface = layer::create(
            &self.layer_shell,
            surface,
            output.as_ref(),
            &self.qh,
            &settings,
        );

        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = self.connection.backend().display_ptr().cast();
        let surface = layer_surface.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let window = Window {
            view,

            output,

            settings,

            width: 0,
            height: 0,

            scale: 1.0,

            frame_requested: false,

            pointer: Pointer::default(),
            on_key: None,

            gpu,

            layer_surface,

            qh: self.qh.clone(),
        };

        self.windows.push(window);
    }
}
