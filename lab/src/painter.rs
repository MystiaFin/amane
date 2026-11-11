use wgpu::{
    ColorTargetState, Device, FragmentState, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, TextureFormat,
    TextureView, VertexState,
};

// draws a window's commands with wgpu
pub struct Painter {
    device: Device,
    queue: Queue,

    boxes: RenderPipeline,
}

impl Painter {
    // every window shares one device, so it's handed in instead of made here
    pub fn new(device: Device, queue: Queue, format: TextureFormat) -> Self {
        let color: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        let mut color_in_bytes: Vec<u8> = Vec::new();

        for value in color {
            let bytes = value.to_ne_bytes();

            color_in_bytes.extend_from_slice(&bytes);
        }

        let shader = device.create_shader_module(wgpu::include_wgsl!("painter/boxes.wgsl"));

        // the shader and its settings, built once so every frame can reuse it
        let boxes = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("boxes"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("place"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("color"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState::from(format))],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            device,
            queue,

            boxes,
        }
    }

    // the target is the window's next picture
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

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(attachment)],
            ..Default::default()
        });

        pass.set_pipeline(&self.boxes);

        // 6 corners make one box
        pass.draw(0..6, 0..1);

        // the pass has to end before the encoder can be finished
        drop(pass);

        self.queue.submit([encoder.finish()]);
    }
}
