use vello::wgpu::{CurrentSurfaceTexture, SurfaceTexture};

use crate::graphics::renderer::Command;

use super::{Gpu, texture};

impl Gpu {
    pub fn draw(&mut self, commands: Vec<Command>, width: u32, height: u32) {
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);
        }

        let Some(frame) = self.next_frame() else {
            return;
        };

        let canvas = texture::canvas(&self.device, width, height);

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

        // presenting attaches the frame to the wayland surface and commits it
        frame.present();
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

            CurrentSurfaceTexture::Validation => panic!("failed to get the next frame"),
        }
    }
}
