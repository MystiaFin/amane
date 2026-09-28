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

        // a handler may have changed a service, so the view has to run again
        if handled {
            self.request_frame();
        }
    }
}

impl WaylandState {
    // returns whether a handler ran
    fn handle_pointer(&mut self, event: &PointerEvent) -> bool {
        let (x, y) = event.position;

        match &event.kind {
            PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                self.pointer.move_to(x as f32, y as f32)
            }

            PointerEventKind::Leave { .. } => self.pointer.leave(),

            PointerEventKind::Press { .. } => {
                self.pointer.press();

                false
            }

            PointerEventKind::Release { button, .. } => {
                // side buttons like back and forward have no amane button yet
                let Some(button) = button::translate(*button) else {
                    return false;
                };

                self.pointer.release(button)
            }

            PointerEventKind::Axis {
                horizontal,
                vertical,
                ..
            } => self.pointer.scroll(scroll::translate(horizontal, vertical)),
        }
    }
}
