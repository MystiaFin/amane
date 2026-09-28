use smithay_client_toolkit::{compositor::FrameCallbackData, shell::WaylandSurface};

use crate::graphics::{Rect, Renderer};

use super::WaylandState;

impl WaylandState {
    pub fn redraw(&mut self) {
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
    }

    pub fn request_frame(&mut self) {
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
