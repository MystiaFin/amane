use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor::WlCompositor, wl_output::WlOutput},
};

use crate::graphics::Gpu;
use crate::InputArea;
use crate::input::{KeyHandler, Pointer};

use super::{WaylandState, settings::Settings, view::View};

// one layer surface on screen, with everything it needs to draw and take input
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

    // both come from the last drawn view, so input matches what is on screen
    pub pointer: Pointer,
    pub on_key: Option<KeyHandler>,

    // the gpu draws into the layer surface, so it has to go first when both are dropped
    pub gpu: Gpu,

    pub layer_surface: LayerSurface,

    // kept to make input regions, which come from the compositor
    pub compositor: WlCompositor,

    pub qh: QueueHandle<WaylandState>,
}
