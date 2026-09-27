use vello::wgpu::util::{BufferInitDescriptor, DeviceExt};
use vello::wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d, FragmentState,
    LoadOp, Operations, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDimension, VertexState, include_wgsl,
};

/*
 * a fragment shader from passes.wgsl run once for every pixel of the target,
 * reading one source texture of the same size and four settings numbers
 */
pub struct Pass {
    pipeline: RenderPipeline,
    inputs: BindGroupLayout,
}

impl Pass {
    pub fn new(
        device: &Device,
        fragment: &str,
        format: TextureFormat,
        blend: Option<BlendState>,
    ) -> Self {
        let inputs = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[source_entry(), settings_entry()],
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&inputs)],
            immediate_size: 0,
        });

        let module = device.create_shader_module(include_wgsl!("passes.wgsl"));

        let target = ColorTargetState {
            format,
            blend,
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
                entry_point: Some(fragment),
                compilation_options: Default::default(),
                targets: &[Some(target)],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self { pipeline, inputs }
    }

    pub fn run(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        source: &TextureView,
        target: &TextureView,
        settings: [f32; 4],
    ) {
        let settings: Vec<u8> = settings
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();

        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &settings,
            usage: BufferUsages::UNIFORM,
        });

        let inputs = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.inputs,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(source),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: settings.as_entire_binding(),
                },
            ],
        });

        // the target keeps what it had, the blend state decides how the new pixels land on it
        let attachment = RenderPassColorAttachment {
            view: target,
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

        // three corners make one triangle that covers the whole target
        pass.draw(0..3, 0..1);
    }
}

fn source_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn settings_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

// what the commands draw onto, one premultiplied color per real pixel
pub fn canvas(device: &Device, width: u32, height: u32) -> Texture {
    let usage =
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;

    create(device, width, height, usage)
}

// vello writes here from a compute shader, with plain, not premultiplied, alpha
pub fn scratch(device: &Device, width: u32, height: u32) -> Texture {
    let usage = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    create(device, width, height, usage)
}

// vello copies the finished blur into its image atlas, so it has to be a copy source
pub fn blur(device: &Device, width: u32, height: u32) -> Texture {
    let usage = TextureUsages::RENDER_ATTACHMENT
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST;

    create(device, width, height, usage)
}

pub fn view(texture: &Texture) -> TextureView {
    texture.create_view(&Default::default())
}

// new textures start out fully transparent
fn create(device: &Device, width: u32, height: u32, usage: TextureUsages) -> Texture {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}
