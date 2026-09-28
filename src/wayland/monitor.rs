use smithay_client_toolkit::output::OutputInfo;

use crate::Monitor;

pub fn describe(info: &OutputInfo) -> Monitor {
    let name = info.name.clone().unwrap_or_default();

    let current = info.modes.iter().find(|mode| mode.current);

    // the logical size already has scaling and rotation applied, the mode does not
    let (width, height) = match (info.logical_size, current) {
        (Some(size), _) => size,
        (None, Some(mode)) => mode.dimensions,
        (None, None) => (0, 0),
    };

    Monitor {
        name,

        width: width as u32,
        height: height as u32,
    }
}
