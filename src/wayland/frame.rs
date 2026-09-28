use smithay_client_toolkit::{compositor::FrameCallbackData, shell::WaylandSurface};

use crate::animation::moving;
use crate::graphics::{Rect, Renderer};

use super::window::Window;

impl Window {
    pub fn redraw(&mut self) {
        // the view runs again on every redraw, so it shows the services as they are now
        let window = self.view.run();

        let moving = moving::take();

        self.update_surface(&window);

        let (width, height) = (self.width, self.height);

        // 0 until the compositor configures the window, and again while it is hidden
        if width == 0 || height == 0 {
            return;
        }

        let Some(root) = window.root else {
            panic!("failed to draw window: no child set");
        };

        // the window is measured in logical pixels, the buffer in real ones
        let buffer_width = width * self.scale as u32;
        let buffer_height = height * self.scale as u32;

        let mut renderer = Renderer::new(self.scale);

        let area = Rect::new(
            0.0,
            0.0,
            root.width().resolve(width as f32),
            root.height().resolve(height as f32),
        );

        root.draw(&mut renderer, area);

        // the handlers are rebuilt with the view, so each frame replaces the last frame's
        self.pointer.collect(root.as_ref(), area);

        self.on_key = window.on_key;

        let surface = self.layer_surface.wl_surface();

        // the scale goes out with the commit that presenting the frame makes
        surface.set_buffer_scale(self.scale as i32);

        self.gpu
            .draw(renderer.finish(), buffer_width, buffer_height);

        /*
         * an animation that has not arrived yet needs the next frame too,
         * a hidden window never gets here so it waits until it shows again
         */
        if moving {
            self.request_frame();
        }
    }

    pub fn request_frame(&mut self) {
        /*
         * a hidden or unconfigured window gets no frame callbacks,
         * so the view runs right away to see if the window should show
         */
        if self.width == 0 {
            self.redraw();

            return;
        }

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
