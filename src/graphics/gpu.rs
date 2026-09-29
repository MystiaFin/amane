mod blur;
mod clip;
mod convert;
mod cut;
mod dispatch;
mod atlas;
mod glyph;
mod gradient;
mod flush;
mod frame;
mod image;
mod layer;
mod paint;
mod pass;
mod picture;
mod quads;
mod shade;
mod shader;
mod shadow;
mod shape;
mod surface;
mod texture;
mod wait;

use std::collections::HashMap;
use std::ffi::c_void;
use std::path::PathBuf;

use vello::peniko::ImageData;
use vello::wgpu::{
    BlendState, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat,
};
use vello::{AaSupport, RendererOptions};

use pass::Pass;
use shader::Shader;
use wait::wait;

/*
 * the only part of amane that knows how drawing is done,
 * vello draws the shapes and wgpu runs everything else on the gpu
 */
pub struct Gpu {
    device: Device,
    queue: Queue,

    surface: Surface<'static>,
    config: SurfaceConfiguration,

    vello: vello::Renderer,

    // images already turned into vello's form, keyed by where the loaded image lives
    images: HashMap<usize, ImageData>,

    // letters already drawn into pictures, none for letters without pixels like a space
    glyphs: HashMap<glyph::GlyphKey, Option<glyph::Glyph>>,

    quads: quads::Quads,
    atlas_dropped: bool,

    // custom shaders, compiled once and kept by the path they were read from
    shaders: HashMap<PathBuf, Shader>,

    composite: Pass,
    present: Pass,
    box_blur: Pass,
    erase: Pass,
}

impl Gpu {
    // the pointers are libwayland's display and surface, which have to outlive the gpu
    pub fn new(display: *mut c_void, surface: *mut c_void) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());

        let surface = surface::create(&instance, display, surface);

        let adapter_options = RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..RequestAdapterOptions::default()
        };

        let adapter = wait(instance.request_adapter(&adapter_options))
            .expect("failed to find a gpu that can draw to the window");

        let (device, queue) =
            wait(adapter.request_device(&DeviceDescriptor::default())).expect("failed to open gpu");

        let config = surface::configure(&surface, &adapter);

        // area anti-aliasing matches how edges looked with the cpu renderer
        let vello_options = RendererOptions {
            antialiasing_support: AaSupport::area_only(),
            ..RendererOptions::default()
        };

        let vello = vello::Renderer::new(&device, vello_options).expect("failed to start vello");

        // the canvas and the surface hold premultiplied colors
        let over = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);

        let composite = Pass::new(&device, "composite", TextureFormat::Rgba8Unorm, over);
        let present = Pass::new(&device, "composite", config.format, None);
        let box_blur = Pass::new(&device, "box_blur", TextureFormat::Rgba8Unorm, None);
        let erase = cut::erase(&device);

        let quads = quads::Quads::new(&device, &queue);

        Self {
            device,
            queue,

            surface,
            config,

            vello,

            images: HashMap::new(),
            glyphs: HashMap::new(),
            quads,
            atlas_dropped: false,

            shaders: HashMap::new(),

            composite,
            present,
            box_blur,
            erase,
        }
    }
}
