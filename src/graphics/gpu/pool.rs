use vello::wgpu::{
    Color, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture,
    TextureUsages,
};

use super::{Gpu, texture};

/*
 * textures the size of the window are needed several times a frame,
 * so finished ones are kept and handed out again instead of allocated
 */
impl Gpu {
    // a reused canvas still holds the last drawing, so it's cleared first
    pub(super) fn take_canvas(&mut self, width: u32, height: u32) -> Texture {
        let Some(canvas) = self.take(width, height, texture::CANVAS) else {
            return texture::canvas(&self.device, width, height);
        };

        self.clear(&canvas);

        canvas
    }

    // vello overwrites every pixel of it, so no clearing needed
    pub(super) fn take_scratch(&mut self, width: u32, height: u32) -> Texture {
        match self.take(width, height, texture::SCRATCH) {
            Some(scratch) => scratch,
            None => texture::scratch(&self.device, width, height),
        }
    }

    pub(super) fn give_back(&mut self, texture: Texture) {
        self.spare.push(texture);
    }

    fn take(&mut self, width: u32, height: u32, usage: TextureUsages) -> Option<Texture> {
        let fits = |texture: &Texture| {
            texture.width() == width && texture.height() == height && texture.usage() == usage
        };

        let index = self.spare.iter().position(fits)?;

        Some(self.spare.swap_remove(index))
    }

    fn clear(&self, canvas: &Texture) {
        let view = texture::view(canvas);

        let attachment = RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::TRANSPARENT),
                store: StoreOp::Store,
            },
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(attachment)],
            ..Default::default()
        });

        self.queue.submit([encoder.finish()]);
    }
}
