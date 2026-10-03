use smithay_client_toolkit::output::{OutputHandler, OutputInfo, OutputState};
use wayland_client::{Connection, QueueHandle, protocol::wl_output::WlOutput};

use crate::Monitor;

use super::{WaylandState, surface::View};

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    // also sent for the monitors that were already there when amane started
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let Some(info) = self.output.info(&output) else {
            return;
        };

        let monitor = describe(&info);

        let views = self.per_monitor.clone();

        for view in views {
            let view = View::Monitor(view, monitor.clone());

            self.open(view, Some(output.clone()));
        }

        // a monitor plugged in while locked needs a lock screen too
        self.open_lock(output);
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let Some(info) = self.output.info(&output) else {
            return;
        };

        let monitor = describe(&info);

        for window in &mut self.windows {
            if window.output.as_ref() != Some(&output) {
                continue;
            }

            if let View::Monitor(_, current) = &mut window.view {
                *current = monitor.clone();
            }

            window.request_frame();
        }
    }

    // dropping a window destroys its layer surface
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        self.windows
            .retain(|window| window.output.as_ref() != Some(&output));
    }
}

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
