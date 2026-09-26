use smithay_client_toolkit::{compositor::FrameCallbackData, shell::WaylandSurface};

use crate::graphics::{Color, Rect, Renderer};

use super::{WaylandState, shm};

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
