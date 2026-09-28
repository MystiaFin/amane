use smithay_client_toolkit::output::{OutputHandler, OutputState};
use wayland_client::{Connection, QueueHandle, protocol::wl_output::WlOutput};

use super::{WaylandState, monitor, view::View};

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    // also sent for the monitors that were already there when amane started
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let Some(info) = self.output.info(&output) else {
            return;
        };

        let monitor = monitor::describe(&info);

        let views = self.per_monitor.clone();

        for view in views {
            let view = View::Monitor(view, monitor.clone());

            self.open(view, Some(output.clone()));
        }
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let Some(info) = self.output.info(&output) else {
            return;
        };

        let monitor = monitor::describe(&info);

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
