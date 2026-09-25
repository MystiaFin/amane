use smithay_client_toolkit::shell::{
    WaylandSurface,
    wlr_layer::{
        self, Anchor, KeyboardInteractivity, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
};
use wayland_client::{Connection, QueueHandle, protocol::wl_surface::WlSurface};

use crate::{Horizontal, Keyboard, Layer, LayerWindow, Vertical, WindowSize, Zone};

use super::WaylandState;

pub fn create(
    layer_shell: &LayerShell,
    surface: WlSurface,
    qh: &QueueHandle<WaylandState>,
    window: &LayerWindow,
) -> LayerSurface {
    // no output given, so the compositor chooses the monitor
    let layer_surface =
        layer_shell.create_layer_surface(qh, surface, layer(window.layer), Some("amane"), None);

    layer_surface.set_size(pixels(window.width), pixels(window.height));

    layer_surface.set_anchor(anchor(window));

    let margin = window.margin;

    layer_surface.set_margin(margin.top, margin.right, margin.bottom, margin.left);

    layer_surface.set_keyboard_interactivity(keyboard(window.keyboard));

    layer_surface.set_exclusive_zone(zone(window));

    layer_surface.commit();

    layer_surface
}

// 0 tells the compositor to stretch between the anchored edges
pub fn pixels(size: WindowSize) -> u32 {
    match size {
        WindowSize::Full => 0,
        WindowSize::Fixed(pixels) => pixels.round() as u32,
    }
}

fn anchor(window: &LayerWindow) -> Anchor {
    let mut anchor = Anchor::empty();

    match window.vertical {
        Vertical::Top => anchor |= Anchor::TOP,
        Vertical::Middle => {}
        Vertical::Bottom => anchor |= Anchor::BOTTOM,
    }

    match window.horizontal {
        Horizontal::Left => anchor |= Anchor::LEFT,
        Horizontal::Middle => {}
        Horizontal::Right => anchor |= Anchor::RIGHT,
    }

    // stretching only works between two opposite anchored edges
    if window.width == WindowSize::Full {
        anchor |= Anchor::LEFT | Anchor::RIGHT;
    }

    if window.height == WindowSize::Full {
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

fn zone(window: &LayerWindow) -> i32 {
    match window.zone {
        Zone::Reserve => reserve(window),
        Zone::Respect => 0,
        Zone::Ignore => -1,
    }
}

// a bar on the left or right edge is as thick as its width, any other bar as its height
fn reserve(window: &LayerWindow) -> i32 {
    let size = match (window.horizontal, window.width) {
        (Horizontal::Left | Horizontal::Right, WindowSize::Fixed(_)) => window.width,
        _ => window.height,
    };

    let reserved = pixels(size);

    i32::try_from(reserved).expect("failed to convert reserved space")
}

impl LayerShellHandler for WaylandState {
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let (width, height) = configure.new_size;

        let width = if width == 0 {
            self.requested_width
        } else {
            width
        };
        let height = if height == 0 {
            self.requested_height
        } else {
            height
        };

        self.width = width;
        self.height = height;

        self.redraw();
    }

    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
}
