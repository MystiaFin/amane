use std::borrow::Cow;
use std::fs;
use std::path::Path;

use vello::wgpu::naga::ShaderStage;
use vello::wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferBindingType, ColorTargetState, ColorWrites, Device, FragmentState,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, VertexState,
};

// amane's vertex half, which also gives wgsl shaders their inputs
const VERTEX: &str = include_str!("shader.wgsl");

const WGSL_INPUTS: &str = "
@group(0) @binding(0) var<uniform> size: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;
@group(0) @binding(3) var<uniform> values: array<vec4<f32>, 16>;
";

const GLSL_INPUTS: &str = "#version 450
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;
layout(set = 0, binding = 0) uniform Size { vec2 size; };
layout(set = 0, binding = 1) uniform Time { float time; };
layout(set = 0, binding = 3) uniform Values { vec4 values[16]; };
";

/*
 * a user's fragment shader over a rectangle: it gets uv from 0 to 1 across the rectangle,
 * size in logical pixels, time in seconds and the rectangle's values (16 is VALUE_ROWS),
 * and returns a plain, not premultiplied, color
 */
pub struct Shader {
    pub pipeline: RenderPipeline,
    pub inputs: BindGroupLayout,
}

impl Shader {
    // .glsl and .frag files are glsl, anything else is wgsl
    pub fn load(device: &Device, path: &Path) -> std::io::Result<Self> {
        let source = fs::read_to_string(path)?;

        let extension = path.extension().and_then(|extension| extension.to_str());

        let is_glsl = matches!(extension, Some("glsl" | "frag"));

        let vertex = module(device, ShaderSource::Wgsl(Cow::Borrowed(VERTEX)));

        let fragment = if is_glsl {
            let shader = format!("{GLSL_INPUTS}{source}");

            module(
                device,
                ShaderSource::Glsl {
                    shader: Cow::Owned(shader),
                    stage: ShaderStage::Fragment,
                    defines: Default::default(),
                },
            )
        } else {
            // the vertex half is added so wgsl sees the same Placement and Corner types
            let shader = format!("{VERTEX}{WGSL_INPUTS}{source}");

            module(device, ShaderSource::Wgsl(Cow::Owned(shader)))
        };

        let inputs = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                uniform(0, ShaderStages::FRAGMENT),
                uniform(1, ShaderStages::FRAGMENT),
                uniform(2, ShaderStages::VERTEX),
                uniform(3, ShaderStages::FRAGMENT),
            ],
        });

        let pipeline = pipeline(device, &inputs, &vertex, &fragment);

        Ok(Self { pipeline, inputs })
    }
}

fn module(device: &Device, source: ShaderSource) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some("custom shader"),
        source,
    })
}

fn pipeline(
    device: &Device,
    inputs: &BindGroupLayout,
    vertex: &ShaderModule,
    fragment: &ShaderModule,
) -> RenderPipeline {
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(inputs)],
        immediate_size: 0,
    });

    // drawn onto a clear texture, so blending turns the plain color into a premultiplied one
    let target = ColorTargetState {
        format: TextureFormat::Rgba8Unorm,
        blend: Some(BlendState::ALPHA_BLENDING),
        write_mask: ColorWrites::ALL,
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: VertexState {
            module: vertex,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(FragmentState {
            module: fragment,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            targets: &[Some(target)],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::gpu::wait::wait;
    use vello::wgpu::{DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterOptions};

    #[test]
    #[ignore = "requires a graphics adapter"]
    fn missing_shader_returns_read_error() {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());
        let adapter = wait(instance.request_adapter(&RequestAdapterOptions::default()))
            .expect("failed to find a gpu");
        let (device, _) =
            wait(adapter.request_device(&DeviceDescriptor::default())).expect("failed to open gpu");

        // A child of a regular file cannot exist, regardless of the working directory.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Cargo.toml")
            .join("missing.wgsl");
        let result = Shader::load(&device, &path);
        assert!(result.is_err(), "an unreadable shader must return an error");
    }
}

fn uniform(binding: u32, visibility: ShaderStages) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
