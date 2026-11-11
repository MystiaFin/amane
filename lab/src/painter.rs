use wgpu::{
    Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    TextureView,
};

// draws a window's commands with wgpu
pub struct Painter {
    device: Device,
    queue: Queue,
}

impl Painter {
    // every window shares one device, so it's handed in instead of made here
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }

    // the target is the window's next picture, or a test's own
    pub fn paint(&self, target: &TextureView) {
        let attachment = RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(wgpu::Color::RED),
                store: StoreOp::Store,
            },
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(attachment)],
            ..Default::default()
        });

        // the pass has to end before the encoder can be finished
        drop(pass);

        self.queue.submit([encoder.finish()]);
    }
}
