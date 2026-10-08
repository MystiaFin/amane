use std::collections::HashMap;

use vello::wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferBindingType, ColorTargetState, ColorWrites, Device, FilterMode, FragmentState,
    MipmapFilterMode, PipelineLayoutDescriptor, Queue, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, Texture, TextureFormat, TextureSampleType,
    TextureViewDimension, VertexState, include_wgsl,
};

use crate::graphics::gpu::atlas::Atlas;
use crate::graphics::gpu::{picture, texture};
use crate::graphics::image::Bitmap;

use super::Quads;

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

        let module = device.create_shader_module(include_wgsl!("../quads.wgsl"));

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
}

pub fn picture_group(
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
