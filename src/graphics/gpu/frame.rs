use vello::wgpu::{CurrentSurfaceTexture, SurfaceTexture};

use crate::graphics::renderer::Command;

use super::{Gpu, texture};

impl Gpu {
    // false when the frame was skipped, so nothing was committed
    pub fn draw(&mut self, commands: Vec<Command>, width: u32, height: u32) -> bool {
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);

            // spares and kept results of the old size would never fit again
            self.spare.clear();
            self.painted.clear();
        }

        let Some(frame) = self.next_frame() else {
            return false;
        };

        self.painting = 0;

        let canvas = self.take_canvas(width, height);

        self.run(commands, &canvas);

        let target = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.present.run(
            &self.device,
            &mut encoder,
            &texture::view(&canvas),
            &target,
            [1.0, 0.0, 0.0, 0.0],
        );

        self.queue.submit([encoder.finish()]);

        self.give_back(canvas);

        self.forget_unshown();

        self.forget_unpainted();

        // presenting attaches the frame to the wayland surface and commits it
        frame.present();

        true
    }

    fn next_frame(&mut self) -> Option<SurfaceTexture> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => Some(frame),

            CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),

            // the swapchain no longer fits the surface, a fresh one takes its place
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);

                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(frame) => Some(frame),

                    _ => None,
                }
            }

            // a hidden or busy window skips this frame, the next redraw catches up
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => None,

            // already reported to the gpu error log, and the next redraw tries again
            CurrentSurfaceTexture::Validation => None,
        }
    }
}
