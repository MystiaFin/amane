use std::any::TypeId;
use std::collections::HashSet;
use std::time::Instant;

use smithay_client_toolkit::{
    session_lock::SessionLockSurface,
    shell::{WaylandSurface, wlr_layer::LayerSurface, xdg::window::Window as XdgWindow},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor::WlCompositor, wl_output::WlOutput, wl_surface::WlSurface},
};

use crate::graphics::Gpu;
use crate::input::{KeyHandler, Pointer};
use crate::{InputArea, LayerWindow, Monitor, Widget, Window};

use super::{
    WaylandState,
    layer::{self, Settings},
    scale::Fractional,
};

// one surface on screen, with everything it needs to draw and take input
pub struct OpenWindow {
    pub view: View,

    // only windows made per monitor are tied to one, the rest let the compositor choose
    pub output: Option<WlOutput>,

    // the input region last sent, so only a real change is sent again
    pub input_region: Option<Vec<InputArea>>,

    pub width: u32,
    pub height: u32,

    pub scale: f32,

    pub frame_requested: bool,

    // when the last frame started drawing, for AMANE_FRAMES
    pub last_frame: Option<Instant>,

    // the services the last view read, a change to any of them draws the window again
    pub reads: HashSet<TypeId>,

    // both come from the last drawn view, so input matches what is on screen
    pub pointer: Pointer,
    pub on_key: Option<KeyHandler>,

    // the gpu draws into the surface, so it has to go first when both are dropped
    pub gpu: Gpu,

    // none at whole scales; dropped before the role, since it belongs to its surface
    pub fractional: Option<Fractional>,

    pub role: Role,

    // kept to make input regions, which come from the compositor
    pub compositor: WlCompositor,

    pub qh: QueueHandle<WaylandState>,
}

// what a window is to the compositor: a layer like a bar, one screen of the session lock, or a normal window
pub enum Role {
    Layer {
        surface: LayerSurface,

        // what the compositor was last told, so only a real change is sent again
        settings: Settings,
    },

    Lock(SessionLockSurface),

    Normal {
        window: XdgWindow,

        // the size it opened at, kept for as long as the compositor leaves the size to it
        size: (u32, u32),
    },
}

// what a window runs on every redraw to find out what it shows
pub enum View {
    Plain(fn() -> LayerWindow),

    // the monitor is replaced when the compositor reports a change to it
    Monitor(fn(&Monitor) -> LayerWindow, Monitor),

    // the name tells it apart from other normal windows
    Normal(&'static str, fn() -> Window),
}

// what a view gave back
pub enum Content {
    Layer(LayerWindow),

    Normal(Window),
}

impl OpenWindow {
    // a size of 0 leaves the choice to the window, which then keeps the size it asked for
    pub fn resize(&mut self, width: u32, height: u32) {
        let (asked_width, asked_height) = self.asked_size();

        self.width = match width {
            0 => asked_width,
            width => width,
        };

        self.height = match height {
            0 => asked_height,
            height => height,
        };

        self.redraw();
    }

    fn asked_size(&self) -> (u32, u32) {
        match &self.role {
            Role::Layer { settings, .. } => {
                let width = layer::to_pixels(settings.width);
                let height = layer::to_pixels(settings.height);

                (width, height)
            }

            Role::Normal { size, .. } => *size,

            // the compositor always gives a lock screen its monitor's size
            Role::Lock(_) => (0, 0),
        }
    }
}

impl Role {
    pub fn wl_surface(&self) -> &WlSurface {
        match self {
            Role::Layer { surface, .. } => surface.wl_surface(),
            Role::Lock(lock_surface) => lock_surface.wl_surface(),
            Role::Normal { window, .. } => window.wl_surface(),
        }
    }

    pub fn commit(&self) {
        self.wl_surface().commit();
    }
}

impl View {
    pub fn run(&self) -> Content {
        match self {
            View::Plain(view) => Content::Layer(view()),
            View::Monitor(view, monitor) => Content::Layer(view(monitor)),
            View::Normal(_, view) => Content::Normal(view()),
        }
    }

    pub fn shows(&self, name: &str) -> bool {
        let View::Normal(shown, _) = self else {
            return false;
        };

        *shown == name
    }
}

impl Content {
    // the name AMANE_FRAMES logs the window under
    pub fn name(&self) -> &'static str {
        match self {
            Content::Layer(window) => window.namespace,
            Content::Normal(_) => "normal",
        }
    }

    // the widgets and the key handler, drawn and used the same way by every kind of window
    pub fn into_parts(self) -> (Option<Box<dyn Widget>>, Option<KeyHandler>) {
        match self {
            Content::Layer(window) => (window.root, window.on_key),
            Content::Normal(window) => (window.root, window.on_key),
        }
    }
}
