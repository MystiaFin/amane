use std::ffi::c_void;
use std::ptr::NonNull;

use wgpu::rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use wgpu::{
    Adapter, CompositeAlphaMode, Device, DeviceDescriptor, Instance, InstanceDescriptor,
    PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceColorSpace, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureUsages,
};

use super::Painter;
use super::wait::wait;

// one window to paint on: it gives the painter the window's next picture, then shows it
pub struct WindowTarget {
    device: Device,
    queue: Queue,

    surface: Surface<'static>,
    config: SurfaceConfiguration,

    painter: Painter,
}

impl WindowTarget {
    // the pointers are libwayland's display and surface, which have to outlive the window target
    pub fn new(display: *mut c_void, surface: *mut c_void) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());

        let surface = create_surface(&instance, display, surface);

        let adapter_options = RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        };

        let adapter = wait(instance.request_adapter(&adapter_options))
            .expect("failed to find a gpu that can draw to the window");

        let (device, queue) =
            wait(adapter.request_device(&DeviceDescriptor::default())).expect("failed to open gpu");

        let config = configure(&surface, &adapter);

        let painter = Painter::new(device.clone(), queue.clone());

        Self {
            device,
            queue,

            surface,
            config,

            painter,
        }
    }

    // false when the frame was skipped, so nothing reached the window
    pub fn draw(&mut self, width: u32, height: u32) -> bool {
        // the window changed size, so the surface has to change with it
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);
        }

        let wgpu::CurrentSurfaceTexture::Success(frame) = self.surface.get_current_texture() else {
            return false;
        };

        let target = frame.texture.create_view(&Default::default());

        self.painter.paint(&target);

        // shows the picture on the window
        self.queue.present(frame);

        true
    }
}

fn create_surface(
    instance: &Instance,
    display: *mut c_void,
    surface: *mut c_void,
) -> Surface<'static> {
    let display = NonNull::new(display).expect("failed to read the wayland display");
    let surface = NonNull::new(surface).expect("failed to read the wayland surface");

    let window = SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            display,
        ))),
        raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface)),
    };

    // the window keeps both pointers alive for longer than the surface
    unsafe { instance.create_surface_unsafe(window) }.expect("failed to create gpu surface")
}

// the size stays 0 until the compositor says how big the window is
fn configure(surface: &Surface, adapter: &Adapter) -> SurfaceConfiguration {
    let abilities = surface.get_capabilities(adapter);

    // an srgb format would change every color on its way to the screen
    let format = abilities
        .formats
        .iter()
        .copied()
        .find(|format| !format.is_srgb())
        .expect("failed to find a surface format without srgb conversion");

    // premultiplied lets the desktop show through see-through pixels
    let alpha_mode = if abilities
        .alpha_modes
        .contains(&CompositeAlphaMode::PreMultiplied)
    {
        CompositeAlphaMode::PreMultiplied
    } else {
        abilities.alpha_modes[0]
    };

    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: SurfaceColorSpace::Auto,
        width: 0,
        height: 0,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: Vec::new(),
    }
}
