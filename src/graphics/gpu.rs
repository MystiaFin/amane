mod blur;
mod paint;
mod pass;

use std::collections::HashMap;
use std::ffi::c_void;
use std::pin::pin;
use std::ptr::NonNull;
use std::task::{Context, Poll, Waker};

use vello::peniko::ImageData;
use vello::wgpu::rwh::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use vello::wgpu::{
    Adapter, BlendState, CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Instance, InstanceDescriptor, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTargetUnsafe, SurfaceTexture, TextureFormat, TextureUsages,
};
use vello::{AaSupport, RendererOptions};

use super::renderer::Command;

use pass::Pass;

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
    atlas_dropped: bool,

    composite: Pass,
    present: Pass,
    box_blur: Pass,
}

impl Gpu {
    // the pointers are libwayland's display and surface, which have to outlive the gpu
    pub fn new(display: *mut c_void, surface: *mut c_void) -> Self {
        let display = NonNull::new(display).expect("failed to read the wayland display");
        let surface = NonNull::new(surface).expect("failed to read the wayland surface");

        let target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                display,
            ))),
            raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface)),
        };

        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());

        // the caller keeps the wayland objects alive for longer than the gpu
        let surface = unsafe { instance.create_surface_unsafe(target) }
            .expect("failed to create gpu surface");

        let adapter_options = RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..RequestAdapterOptions::default()
        };

        let adapter = wait(instance.request_adapter(&adapter_options))
            .expect("failed to find a gpu that can draw to the window");

        let (device, queue) =
            wait(adapter.request_device(&DeviceDescriptor::default())).expect("failed to open gpu");

        let config = configure(&surface, &adapter);

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

        Self {
            device,
            queue,

            surface,
            config,

            vello,

            images: HashMap::new(),
            atlas_dropped: false,

            composite,
            present,
            box_blur,
        }
    }

    pub fn draw(&mut self, commands: Vec<Command>, width: u32, height: u32) {
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);
        }

        let Some(frame) = self.next_frame() else {
            return;
        };

        let canvas = pass::canvas(&self.device, width, height);

        self.run(commands, &canvas);

        let target = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.present.run(
            &self.device,
            &mut encoder,
            &pass::view(&canvas),
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

// the size stays 0 until the compositor says how big the window is
fn configure(surface: &Surface, adapter: &Adapter) -> SurfaceConfiguration {
    let capabilities = surface.get_capabilities(adapter);

    // the canvas already holds display ready values, an srgb format would convert them twice
    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(|format| {
            matches!(
                format,
                TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm
            )
        })
        .expect("failed to find a surface format without srgb conversion");

    // premultiplied lets the compositor show the desktop through transparent pixels
    let alpha_mode = if capabilities
        .alpha_modes
        .contains(&CompositeAlphaMode::PreMultiplied)
    {
        CompositeAlphaMode::PreMultiplied
    } else {
        capabilities.alpha_modes[0]
    };

    // amane waits for frame callbacks itself, so presenting should not wait again
    let present_mode = if capabilities.present_modes.contains(&PresentMode::Mailbox) {
        PresentMode::Mailbox
    } else {
        PresentMode::Fifo
    };

    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: 0,
        height: 0,
        present_mode,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: Vec::new(),
    }
}

// wgpu's native futures are already done when they are made, so one poll finishes them
fn wait<T>(future: impl Future<Output = T>) -> T {
    let mut context = Context::from_waker(Waker::noop());

    let Poll::Ready(value) = pin!(future).poll(&mut context) else {
        panic!("failed to wait for the gpu");
    };

    value
}
