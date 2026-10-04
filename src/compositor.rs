mod hyprland;
mod niri;
mod sway;

use std::env;

use crate::Workspace;

enum Compositor {
    Niri,
    Hyprland,
    Sway,
}

// each compositor sets its own variable for the programs it starts
fn running() -> Option<Compositor> {
    if env::var_os("NIRI_SOCKET").is_some() {
        return Some(Compositor::Niri);
    }

    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return Some(Compositor::Hyprland);
    }

    if env::var_os("SWAYSOCK").is_some() {
        return Some(Compositor::Sway);
    }

    None
}

/*
 * hands over the full workspace list after every compositor event,
 * until the compositor exits; returns right away on an unsupported one
 */
pub fn listen(on_change: impl FnMut(Vec<Workspace>)) {
    let Some(compositor) = running() else {
        return;
    };

    match compositor {
        Compositor::Niri => niri::listen(on_change),
        Compositor::Hyprland => hyprland::listen(on_change),
        Compositor::Sway => sway::listen(on_change),
    }
}

pub fn focus_workspace(id: i64) {
    let Some(compositor) = running() else {
        return;
    };

    match compositor {
        Compositor::Niri => niri::focus_workspace(id),
        Compositor::Hyprland => hyprland::focus_workspace(id),
        Compositor::Sway => sway::focus_workspace(id),
    }
}
