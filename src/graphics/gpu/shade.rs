use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Instant;

use vello::wgpu::util::{BufferInitDescriptor, DeviceExt};
use vello::wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture,
};

use crate::graphics::{Path as Outline, Rect, Transform, VALUE_ROWS};

use super::shader::Shader;
use super::{Gpu, texture};

// every shader's time counts from when the first one was drawn
static START: LazyLock<Instant> = LazyLock::new(Instant::now);

impl Gpu {
    /*
     * the shader draws on a canvas of its own, which is trimmed to the rounded
     * outline and laid over what the commands before it drew
     */
    pub(super) fn shade(
        &mut self,
        canvas: &Texture,
        shader: &Path,
        values: &[[f32; 4]],
        rect: Rect,
        outline: &Outline,
        transform: Transform,
    ) {
        let layer = texture::canvas(&self.device, canvas.width(), canvas.height());

        self.draw_shader(&layer, shader, values, rect, transform);

        self.trim(&layer, outline, transform);

        self.lay(&layer, canvas, 1.0, false);
    }

    fn draw_shader(
        &mut self,
        layer: &Texture,
        shader: &Path,
        values: &[[f32; 4]],
        rect: Rect,
        transform: Transform,
    ) {
        // compiled the first time a path is drawn, then kept
        if !self.shaders.contains_key(shader) {
            let compiled = Shader::load(&self.device, shader);

            self.shaders.insert(PathBuf::from(shader), compiled);
        }

        let shader = &self.shaders[shader];

        let time = START.elapsed().as_secs_f32();

        let size = self.uniform([rect.width, rect.height, 0.0, 0.0]);
        let time = self.uniform([time, 0.0, 0.0, 0.0]);

        let placement = self.uniform_rows([
            [rect.x, rect.y, rect.width, rect.height],
            [transform.sx, transform.kx, transform.tx, 0.0],
            [transform.ky, transform.sy, transform.ty, 0.0],
            [layer.width() as f32, layer.height() as f32, 0.0, 0.0],
        ]);

        // the shader always reads every row, so the ones left out are 0
        let mut rows = [[0.0; 4]; VALUE_ROWS];

        for (row, value) in rows.iter_mut().zip(values) {
            *row = *value;
        }

        let values = self.uniform_rows(rows);

        let inputs = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &shader.inputs,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: size.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: time.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: placement.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: values.as_entire_binding(),
                },
            ],
        });

        let target = texture::view(layer);

        let attachment = RenderPassColorAttachment {
            view: &target,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(attachment)],
                ..Default::default()
            });

            pass.set_pipeline(&shader.pipeline);

            pass.set_bind_group(0, &inputs, &[]);

            // the six corners of the rectangle's two triangles
            pass.draw(0..6, 0..1);
        }

        self.queue.submit([encoder.finish()]);
    }

    fn uniform(&self, values: [f32; 4]) -> Buffer {
        self.uniform_rows([values])
    }

    fn uniform_rows<const ROWS: usize>(&self, rows: [[f32; 4]; ROWS]) -> Buffer {
        let mut bytes = Vec::new();

        for row in rows {
            for value in row {
                bytes.extend(value.to_ne_bytes());
            }
        }

        self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &bytes,
            usage: BufferUsages::UNIFORM,
        })
    }
}
