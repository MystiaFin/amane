use std::collections::HashSet;

use wayland_client::{Proxy, protocol::wl_output::WlOutput};

use crate::graphics::Gpu;
use crate::input::Pointer;
use crate::services::wake;

use super::{WaylandState, layer, role::Role, settings::Settings, view::View, window::Window};

impl WaylandState {
    pub fn open(&mut self, view: View, output: Option<WlOutput>) {
        let surface = self.compositor.create_surface(&self.qh);

        // anything read before belongs to another window
        wake::take_read();

        // later views can change these, each redraw compares them with the last ones
        let window = view.run();

        /*
         * a window that starts hidden never gets a configure, so it never
         * draws; what its first view read is all that can wake it to show
         */
        let reads = wake::take_read();

        let settings = Settings::from(&window);

        let layer_surface = layer::create(
            &self.layer_shell,
            surface,
            output.as_ref(),
            &self.qh,
            &settings,
        );

        self.add(view, output, settings, Role::Layer(layer_surface));

        if let Some(window) = self.windows.last_mut() {
            window.reads = reads;
        }
    }

    // everything a window needs besides its surface is the same for layers and lock screens
    pub fn add(&mut self, view: View, output: Option<WlOutput>, settings: Settings, role: Role) {
        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = self.connection.backend().display_ptr().cast();
        let surface = role.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let window = Window {
            view,

            output,

            settings,

            input_region: None,

            width: 0,
            height: 0,

            scale: 1.0,

            frame_requested: false,
            last_frame: None,
            reads: HashSet::new(),

            pointer: Pointer::default(),
            on_key: None,

            gpu,

            role,

            compositor: self.compositor.wl_compositor().clone(),

            qh: self.qh.clone(),
        };

        self.windows.push(window);
    }
}
