use std::collections::HashMap;

use vello::wgpu::util::{BufferInitDescriptor, DeviceExt};
use vello::wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, Device, FilterMode,
    FragmentState, LoadOp, MipmapFilterMode, Operations, PipelineLayoutDescriptor, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp, Texture, TextureFormat,
    TextureSampleType, TextureViewDimension, VertexState, include_wgsl,
};

use ttf_parser::Face;

use crate::graphics::image::Bitmap;
use crate::graphics::{Color, Rect};

use super::atlas::Atlas;
use super::{picture, texture};

// how many numbers one quad takes in quads.wgsl
const QUAD_SIZE: usize = 28;

// what kind of quad it is, as quads.wgsl tells them apart
const SHAPE: f32 = 0.0;
const LETTER: f32 = 1.0;
const PICTURE: f32 = 2.0;

// nothing clips: far bigger than any canvas
const NO_CLIP: Clip = Clip {
    rect: Rect::new(-1.0e6, -1.0e6, 2.0e6, 2.0e6),
    radius: 0.0,
};

// a rounded rectangle on the canvas that quads only show inside
#[derive(Clone, Copy)]
pub struct Clip {
    pub rect: Rect,
    pub radius: f32,
}

// at most two rounded clips reach a quad, the outer one and the one inside it
#[derive(Clone, Copy, Default)]
pub struct Clips {
    pub outer: Option<Clip>,
    pub inner: Option<Clip>,
}

impl Clips {
    // a clip inside the ones already there; with two there already, the outer two merge
    pub fn within(self, clip: Clip) -> Clips {
        match (self.outer, self.inner) {
            (None, _) => Clips {
                outer: Some(clip),
                inner: None,
            },

            (Some(outer), None) => Clips {
                outer: Some(outer),
                inner: Some(clip),
            },

            (Some(outer), Some(inner)) => Clips {
                outer: Some(narrow(outer, inner)),
                inner: Some(clip),
            },
        }
    }
}

// quads next to each other that read the same image, drawn in one go
struct Run {
    // where the image lives, none for quads that read no image
    picture: Option<usize>,

    first: u32,
    count: u32,
}

/*
 * rounded rectangles, borders, letters and images gathered as they come and
 * drawn in one pass; this is most of what a shell shows, and the gpu draws
 * it far faster as plain triangles than vello does as vector shapes
 */
pub struct Quads {
    pipeline: RenderPipeline,
    inputs: BindGroupLayout,
    picture_inputs: BindGroupLayout,

    sampler: Sampler,

    atlas: Atlas,

    // images already on the gpu, keyed by where the loaded image lives
    pictures: HashMap<usize, BindGroup>,

    // what quads that read no image bind in the image's place
    blank: BindGroup,

    // every quad's numbers one after another, as quads.wgsl reads them
    waiting: Vec<f32>,
    runs: Vec<Run>,
}

impl Quads {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let inputs = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[quads_entry(), canvas_entry(), texture_entry(2, false)],
        });

        let picture_inputs = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[texture_entry(0, true), sampler_entry()],
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&inputs), Some(&picture_inputs)],
            immediate_size: 0,
        });

        let module = device.create_shader_module(include_wgsl!("quads.wgsl"));

        // the canvas holds premultiplied colors, so the quads land on it the same way
        let target = ColorTargetState {
            format: TextureFormat::Rgba8Unorm,
            blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &module,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(target)],
            }),
            multiview_mask: None,
            cache: None,
        });

        // smooth between pixels and between the prepared sizes
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Linear,
            ..Default::default()
        });

        let empty = Bitmap::empty();
        let empty = picture::upload(device, queue, &empty);

        let blank = picture_group(device, &picture_inputs, &empty, &sampler);

        Self {
            pipeline,
            inputs,
            picture_inputs,
            sampler,
            atlas: Atlas::new(device),
            pictures: HashMap::new(),
            blank,
            waiting: Vec::new(),
            runs: Vec::new(),
        }
    }

    // rect and radius in canvas pixels
    pub fn rectangle(&mut self, rect: Rect, radius: f32, color: Color, clips: Clips) {
        self.push(rect, [radius, 0.0], color, clips, SHAPE, [0.0; 4], None);
    }

    pub fn border(
        &mut self,
        rect: Rect,
        radius: f32,
        thickness: f32,
        color: Color,
        clips: Clips,
    ) {
        self.push(
            rect,
            [radius, thickness],
            color,
            clips,
            SHAPE,
            [0.0; 4],
            None,
        );
    }

    // x and y are where the letter's baseline starts, on a whole pixel
    #[allow(clippy::too_many_arguments)]
    pub fn letter(
        &mut self,
        queue: &Queue,
        face: &'static Face<'static>,
        id: u16,
        size: f32,
        color: Color,
        x: f32,
        y: f32,
        clips: Clips,
    ) {
        let Some(letter) = self.atlas.letter(queue, face, id, size) else {
            return;
        };

        let rect = Rect::new(
            x + letter.left,
            y + letter.top,
            letter.width as f32,
            letter.height as f32,
        );

        let texels = [
            letter.x as f32,
            letter.y as f32,
            (letter.x + letter.width) as f32,
            (letter.y + letter.height) as f32,
        ];

        self.push(rect, [0.0; 2], color, clips, LETTER, texels, None);
    }

    // the whole image stretched over rect, which the clip usually trims
    pub fn picture(
        &mut self,
        device: &Device,
        queue: &Queue,
        image: &'static Bitmap,
        rect: Rect,
        clips: Clips,
    ) {
        let key = std::ptr::from_ref(image) as usize;

        if !self.pictures.contains_key(&key) {
            let texture = picture::upload(device, queue, image);

            let group = picture_group(device, &self.picture_inputs, &texture, &self.sampler);

            self.pictures.insert(key, group);
        }

        let corners = [0.0, 0.0, 1.0, 1.0];

        self.push(
            rect,
            [0.0; 2],
            Color::WHITE,
            clips,
            PICTURE,
            corners,
            Some(key),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        rect: Rect,
        [radius, thickness]: [f32; 2],
        color: Color,
        clips: Clips,
        kind: f32,
        source: [f32; 4],
        picture: Option<usize>,
    ) {
        let outer = clips.outer.unwrap_or(NO_CLIP);
        let inner = clips.inner.unwrap_or(NO_CLIP);

        let channel = |value: u8| f32::from(value) / 255.0;

        let index = (self.waiting.len() / QUAD_SIZE) as u32;

        self.waiting.extend([
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            outer.rect.x,
            outer.rect.y,
            outer.rect.width,
            outer.rect.height,
            channel(color.r),
            channel(color.g),
            channel(color.b),
            channel(color.a),
            radius,
            thickness,
            outer.radius,
            kind,
        ]);

        self.waiting.extend(source);

        self.waiting.extend([
            inner.rect.x,
            inner.rect.y,
            inner.rect.width,
            inner.rect.height,
            inner.radius,
            0.0,
            0.0,
            0.0,
        ]);

        // a quad reading the same image as the one before joins its run
        if let Some(run) = self.runs.last_mut()
            && run.picture == picture
        {
            run.count += 1;

            return;
        }

        self.runs.push(Run {
            picture,
            first: index,
            count: 1,
        });
    }

    // draws everything gathered onto the canvas, over what is already there
    pub fn draw(&mut self, device: &Device, queue: &Queue, canvas: &Texture) {
        if self.waiting.is_empty() {
            return;
        }

        let numbers: Vec<u8> = self
            .waiting
            .drain(..)
            .flat_map(|value| value.to_ne_bytes())
            .collect();

        let quads = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &numbers,
            usage: BufferUsages::STORAGE,
        });

        let size: Vec<u8> = [canvas.width() as f32, canvas.height() as f32, 0.0, 0.0]
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();

        let size = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &size,
            usage: BufferUsages::UNIFORM,
        });

        let atlas = texture::view(self.atlas.texture());

        let inputs = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.inputs,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: quads.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: size.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&atlas),
                },
            ],
        });

        let target = texture::view(canvas);

        let mut encoder = device.create_command_encoder(&Default::default());

        {
            let attachment = RenderPassColorAttachment {
                view: &target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            };

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(attachment)],
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);

            pass.set_bind_group(0, &inputs, &[]);

            for run in self.runs.drain(..) {
                let picture = run
                    .picture
                    .and_then(|key| self.pictures.get(&key))
                    .unwrap_or(&self.blank);

                pass.set_bind_group(1, picture, &[]);

                // six corners make each quad's two triangles
                pass.draw(0..6, run.first..run.first + run.count);
            }
        }

        queue.submit([encoder.finish()]);
    }
}

fn picture_group(
    device: &Device,
    layout: &BindGroupLayout,
    texture: &Texture,
    sampler: &Sampler,
) -> BindGroup {
    let view = texture::view(texture);

    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn quads_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX_FRAGMENT,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn canvas_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, filterable: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    }
}

pub fn narrow(outer: Clip, inner: Clip) -> Clip {
    let left = f32::max(outer.rect.x, inner.rect.x);
    let top = f32::max(outer.rect.y, inner.rect.y);

    let outer_right = outer.rect.x + outer.rect.width;
    let outer_bottom = outer.rect.y + outer.rect.height;

    let right = f32::min(outer_right, inner.rect.x + inner.rect.width);
    let bottom = f32::min(outer_bottom, inner.rect.y + inner.rect.height);

    let width = f32::max(right - left, 0.0);
    let height = f32::max(bottom - top, 0.0);

    Clip {
        rect: Rect::new(left, top, width, height),
        radius: inner.radius,
    }
}
