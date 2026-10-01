use std::any::TypeId;
use std::collections::HashSet;
use std::time::Instant;

use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor::WlCompositor, wl_output::WlOutput},
};

use crate::graphics::Gpu;
use crate::InputArea;
use crate::input::{KeyHandler, Pointer};

use super::{WaylandState, layer, role::Role, settings::Settings, view::View};

// one surface on screen, with everything it needs to draw and take input
pub struct Window {
    pub view: View,

    // only windows made per monitor are tied to one, the rest let the compositor choose
    pub output: Option<WlOutput>,

    // what the compositor was last told, so only a real change is sent again
    pub settings: Settings,

    // the input region is not Copy like the rest, so it is kept on its own
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

    pub role: Role,

    // kept to make input regions, which come from the compositor
    pub compositor: WlCompositor,

    pub qh: QueueHandle<WaylandState>,
}

impl Window {
    // a size of 0 leaves the choice to the window, which then keeps the size it asked for
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = match width {
            0 => layer::pixels(self.settings.width),
            width => width,
        };

        self.height = match height {
            0 => layer::pixels(self.settings.height),
            height => height,
        };

        self.redraw();
    }
}
