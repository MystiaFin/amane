use std::collections::HashSet;

use wayland_client::{Proxy, protocol::{wl_output::WlOutput, wl_surface::WlSurface}};

use crate::graphics::Gpu;
use crate::input::Pointer;
use crate::{changes, frame};

use super::{WaylandState, layer::{self, Settings}, surface::{Content, OpenWindow, Role, View}};

impl WaylandState {
    pub fn open(&mut self, view: View, output: Option<WlOutput>) {
        let surface = self.compositor.create_surface(&self.qh);

        /*
         * later views can change the settings, each redraw compares them with the
         * last ones; a window that starts hidden never gets a configure, so it never
         * draws, and what its first view read is all that can wake it to show
         */
        let (content, reads) = frame::run_view(|| view.run(), 0, 0);

        // normal windows open in open_normal
        let Content::Layer(window) = content else {
            unreachable!("only layer views open as layer windows");
        };

        let settings = Settings::from(&window);

        let layer_surface = layer::create(
            &self.layer_shell,
            surface,
            output.as_ref(),
            &self.qh,
            &settings,
        );

        let role = Role::Layer {
            surface: layer_surface,
            settings,
        };

        self.add(view, output, role);

        if let Some(window) = self.windows.last_mut() {
            window.reads = reads;
        }
    }

    // everything a window needs besides its surface is the same for layers and lock screens
    pub fn add(&mut self, view: View, output: Option<WlOutput>, role: Role) {
        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = self.connection.backend().display_ptr().cast();
        let surface = role.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let fractional = self.make_fractional(role.wl_surface());

        let window = OpenWindow {
            view,

            output,

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

            fractional,

            role,

            compositor: self.compositor.wl_compositor().clone(),

            qh: self.qh.clone(),
        };

        self.windows.push(window);
    }

    // wayland events name the surface they are about, this finds its window
    pub fn window(&mut self, surface: &WlSurface) -> Option<&mut OpenWindow> {
        self.windows
            .iter_mut()
            .find(|window| window.role.wl_surface() == surface)
    }

    // a service or handler may have changed what any of the windows shows
    pub fn request_frames(&mut self) {
        for window in &mut self.windows {
            window.request_frame();
        }
    }

    // input only changes the window it arrived in, besides the services it wrote
    pub fn request_frame_on(&mut self, surface: &WlSurface) {
        if let Some(window) = self.window(surface) {
            window.request_frame();
        }
    }

    // only windows that read a service that changed draw again
    pub fn request_changed_frames(&mut self) {
        let Some(changed) = changes::take() else {
            self.request_frames();

            return;
        };

        for window in &mut self.windows {
            if window.reads.is_disjoint(&changed) {
                continue;
            }

            window.request_frame();
        }
    }

    pub fn close(&mut self, surface: &WlSurface) {
        self.windows
            .retain(|window| window.role.wl_surface() != surface);

        // windows made per monitor come back when a monitor is plugged in again
        if self.windows.is_empty() && self.per_monitor.is_empty() {
            self.running = false;
        }
    }
}
