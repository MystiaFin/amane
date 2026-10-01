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

use super::{WaylandState, layer, role::Role, scale::Fractional, view::View};

// one surface on screen, with everything it needs to draw and take input
pub struct Surface {
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

impl Surface {
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
