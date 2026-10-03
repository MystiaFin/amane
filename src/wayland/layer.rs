use smithay_client_toolkit::shell::{
    WaylandSurface,
    wlr_layer::{
        self, Anchor, KeyboardInteractivity, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
};

use crate::{Horizontal, Keyboard, Layer, LayerWindow, Margin, Vertical, WindowSize, Zone};

use super::WaylandState;

// the parts of the window the compositor knows about, kept to see what the next view changes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub width: WindowSize,
    pub height: WindowSize,

    pub vertical: Vertical,
    pub horizontal: Horizontal,

    pub margin: Margin,
    pub layer: Layer,
    pub keyboard: Keyboard,
    pub zone: Zone,

    pub namespace: &'static str,

    pub visible: bool,
}

impl From<&LayerWindow> for Settings {
    fn from(window: &LayerWindow) -> Self {
        Self {
            width: window.width,
            height: window.height,

            vertical: window.vertical,
            horizontal: window.horizontal,

            margin: window.margin,
            layer: window.layer,
            keyboard: window.keyboard,
            zone: window.zone,

            namespace: window.namespace,

            visible: window.visible,
        }
    }
}

impl LayerShellHandler for WaylandState {
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer_surface: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let Some(window) = self.window(layer_surface.wl_surface()) else {
            return;
        };

        let (width, height) = configure.new_size;

        window.resize(width, height);
    }

    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, layer_surface: &LayerSurface) {
        self.close(layer_surface.wl_surface());
    }
}

pub fn create(
    layer_shell: &LayerShell,
    surface: WlSurface,
    output: Option<&WlOutput>,
    qh: &QueueHandle<WaylandState>,
    settings: &Settings,
) -> LayerSurface {
    // with no output given, the compositor chooses the monitor
    let layer_surface = layer_shell.create_layer_surface(
        qh,
        surface,
        to_layer(settings.layer),
        Some(settings.namespace),
        output,
    );

    apply(&layer_surface, settings);

    // a window that starts hidden makes this first commit once it is shown
    if settings.visible {
        layer_surface.commit();
    }

    layer_surface
}

// the requests only take effect with the next commit
pub fn apply(layer_surface: &LayerSurface, settings: &Settings) {
    layer_surface.set_layer(to_layer(settings.layer));

    layer_surface.set_size(to_pixels(settings.width), to_pixels(settings.height));

    layer_surface.set_anchor(to_anchor(settings));

    let margin = settings.margin;

    layer_surface.set_margin(margin.top, margin.right, margin.bottom, margin.left);

    layer_surface.set_keyboard_interactivity(to_interactivity(settings.keyboard));

    layer_surface.set_exclusive_zone(to_exclusive_zone(settings));
}

// 0 tells the compositor to stretch between the anchored edges
pub fn to_pixels(size: WindowSize) -> u32 {
    match size {
        WindowSize::Full => 0,
        WindowSize::Fixed(pixels) => pixels.round() as u32,
    }
}

fn to_anchor(settings: &Settings) -> Anchor {
    let mut anchor = Anchor::empty();

    match settings.vertical {
        Vertical::Top => anchor |= Anchor::TOP,
        Vertical::Middle => {}
        Vertical::Bottom => anchor |= Anchor::BOTTOM,
    }

    match settings.horizontal {
        Horizontal::Left => anchor |= Anchor::LEFT,
        Horizontal::Middle => {}
        Horizontal::Right => anchor |= Anchor::RIGHT,
    }

    // stretching only works between two opposite anchored edges
    if settings.width == WindowSize::Full {
        anchor |= Anchor::LEFT | Anchor::RIGHT;
    }

    if settings.height == WindowSize::Full {
        anchor |= Anchor::TOP | Anchor::BOTTOM;
    }

    anchor
}

fn to_layer(layer: Layer) -> wlr_layer::Layer {
    match layer {
        Layer::Background => wlr_layer::Layer::Background,
        Layer::Bottom => wlr_layer::Layer::Bottom,
        Layer::Top => wlr_layer::Layer::Top,
        Layer::Overlay => wlr_layer::Layer::Overlay,
    }
}

fn to_interactivity(keyboard: Keyboard) -> KeyboardInteractivity {
    match keyboard {
        Keyboard::None => KeyboardInteractivity::None,
        Keyboard::Exclusive => KeyboardInteractivity::Exclusive,
        Keyboard::OnDemand => KeyboardInteractivity::OnDemand,
    }
}

fn to_exclusive_zone(settings: &Settings) -> i32 {
    match settings.zone {
        Zone::Reserve => measure_thickness(settings),
        Zone::Respect => 0,
        Zone::Ignore => -1,
    }
}

// a bar on the left or right edge is as thick as its width, any other bar as its height
fn measure_thickness(settings: &Settings) -> i32 {
    let size = match (settings.horizontal, settings.width) {
        (Horizontal::Left | Horizontal::Right, WindowSize::Fixed(_)) => settings.width,
        _ => settings.height,
    };

    let reserved = to_pixels(size);

    i32::try_from(reserved).expect("failed to convert reserved space")
}
