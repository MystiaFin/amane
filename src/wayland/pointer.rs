use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use wayland_client::{Connection, QueueHandle, protocol::wl_pointer::WlPointer};

use super::{WaylandState, button, scroll};

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer,
        events: &[PointerEvent],
    ) {
        let mut handled = false;

        for event in events {
            handled |= self.handle_pointer(event);
        }

        // a handler may have changed a service that any window shows
        if handled {
            self.request_frames();
        }
    }
}

impl WaylandState {
    // returns whether a handler ran
    fn handle_pointer(&mut self, event: &PointerEvent) -> bool {
        // every event names the surface it happened on
        let Some(window) = self.window(&event.surface) else {
            return false;
        };

        let pointer = &mut window.pointer;

        let (x, y) = event.position;

        match &event.kind {
            PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                pointer.move_to(x as f32, y as f32)
            }

            PointerEventKind::Leave { .. } => pointer.leave(),

            PointerEventKind::Press { .. } => {
                pointer.press();

                false
            }

            PointerEventKind::Release { button, .. } => {
                // side buttons like back and forward have no amane button yet
                let Some(button) = button::translate(*button) else {
                    return false;
                };

                pointer.release(button)
            }

            PointerEventKind::Axis {
                horizontal,
                vertical,
                ..
            } => pointer.scroll(scroll::translate(horizontal, vertical)),
        }
    }
}
