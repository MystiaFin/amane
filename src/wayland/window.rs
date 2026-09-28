use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_client::{QueueHandle, protocol::wl_output::WlOutput};

use crate::graphics::Gpu;
use crate::input::{KeyHandler, Pointer};

use super::{WaylandState, settings::Settings, view::View};

// one layer surface on screen, with everything it needs to draw and take input
pub struct Window {
    pub view: View,

    // only windows made per monitor are tied to one, the rest let the compositor choose
    pub output: Option<WlOutput>,

    // what the compositor was last told, so only a real change is sent again
    pub settings: Settings,

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

    pub qh: QueueHandle<WaylandState>,
}
