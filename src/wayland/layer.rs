use smithay_client_toolkit::shell::{
    WaylandSurface,
    wlr_layer::{self, Anchor, KeyboardInteractivity, LayerShell, LayerSurface},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
};

use crate::{Horizontal, Keyboard, Layer, Vertical, WindowSize, Zone};

use super::{WaylandState, settings::Settings};

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
        layer(settings.layer),
        Some("amane"),
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
    layer_surface.set_layer(layer(settings.layer));

    layer_surface.set_size(pixels(settings.width), pixels(settings.height));

    layer_surface.set_anchor(anchor(settings));

    let margin = settings.margin;

    layer_surface.set_margin(margin.top, margin.right, margin.bottom, margin.left);

    layer_surface.set_keyboard_interactivity(keyboard(settings.keyboard));

    layer_surface.set_exclusive_zone(zone(settings));
}

// 0 tells the compositor to stretch between the anchored edges
pub fn pixels(size: WindowSize) -> u32 {
    match size {
        WindowSize::Full => 0,
        WindowSize::Fixed(pixels) => pixels.round() as u32,
    }
}

fn anchor(settings: &Settings) -> Anchor {
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

fn layer(layer: Layer) -> wlr_layer::Layer {
    match layer {
        Layer::Background => wlr_layer::Layer::Background,
        Layer::Bottom => wlr_layer::Layer::Bottom,
        Layer::Top => wlr_layer::Layer::Top,
        Layer::Overlay => wlr_layer::Layer::Overlay,
    }
}

fn keyboard(keyboard: Keyboard) -> KeyboardInteractivity {
    match keyboard {
        Keyboard::None => KeyboardInteractivity::None,
        Keyboard::Exclusive => KeyboardInteractivity::Exclusive,
        Keyboard::OnDemand => KeyboardInteractivity::OnDemand,
    }
}

fn zone(settings: &Settings) -> i32 {
    match settings.zone {
        Zone::Reserve => reserve(settings),
        Zone::Respect => 0,
        Zone::Ignore => -1,
    }
}

// a bar on the left or right edge is as thick as its width, any other bar as its height
fn reserve(settings: &Settings) -> i32 {
    let size = match (settings.horizontal, settings.width) {
        (Horizontal::Left | Horizontal::Right, WindowSize::Fixed(_)) => settings.width,
        _ => settings.height,
    };

    let reserved = pixels(size);

    i32::try_from(reserved).expect("failed to convert reserved space")
}
