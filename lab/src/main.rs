mod painter;
mod wait;
mod window_target;

use std::ffi::c_void;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::{Window, WindowId};

use painter::Painter;
use window_target::WindowTarget;

// one plain window showing what the painter draws, and nothing else of amane
#[derive(Default)]
struct Lab {
    // the target draws into the window, so it has to be dropped first
    target: Option<WindowTarget>,
    window: Option<Window>,
}

impl ApplicationHandler for Lab {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes().with_title("lab");

        let window = event_loop
            .create_window(attributes)
            .expect("failed to open the window");

        let (display, surface) = wayland_pointers(&window);

        self.target = Some(WindowTarget::new(display, surface));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.redraw(),

            _ => {}
        }
    }
}

impl Lab {
    fn redraw(&mut self) {
        let (Some(target), Some(window)) = (&mut self.target, &self.window) else {
            return;
        };

        let size = window.inner_size();

        // the surface can't be set up at a size of 0
        if size.width == 0 || size.height == 0 {
            return;
        }

        target.draw(size.width, size.height);
    }
}

// the window's libwayland display and surface, the same two pointers amane hands the painter
fn wayland_pointers(window: &Window) -> (*mut c_void, *mut c_void) {
    let display = window
        .display_handle()
        .expect("failed to read the display")
        .as_raw();

    let surface = window
        .window_handle()
        .expect("failed to read the window")
        .as_raw();

    let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(surface)) =
        (display, surface)
    else {
        panic!("failed to open a wayland window: the lab only runs on wayland");
    };

    (display.display.as_ptr(), surface.surface.as_ptr())
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to start the event loop");

    let mut lab = Lab::default();

    event_loop.run_app(&mut lab).expect("failed to run the lab");
}
