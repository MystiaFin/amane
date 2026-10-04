mod niri;

use std::env;

use crate::Workspace;

enum Compositor {
    Niri,
}

// each compositor sets its own variable for the programs it starts
fn running() -> Option<Compositor> {
    if env::var_os("NIRI_SOCKET").is_some() {
        return Some(Compositor::Niri);
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
    }
}

pub fn focus_workspace(id: u64) {
    let Some(compositor) = running() else {
        return;
    };

    match compositor {
        Compositor::Niri => niri::focus_workspace(id),
    }
}
