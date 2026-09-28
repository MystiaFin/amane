use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use wayland_client::{Connection, QueueHandle, protocol::wl_pointer::WlPointer};

use crate::Button;

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
                let hovered = pointer.move_to(x as f32, y as f32);
                let dragged = pointer.drag();
                let moved = pointer.report_motion();

                let cursor = pointer.cursor();

                self.show_cursor(cursor);

                hovered || dragged || moved
            }

            PointerEventKind::Leave { .. } => {
                let left = pointer.leave();

                self.forget_cursor();

                left
            }

            PointerEventKind::Press { button, .. } => {
                pointer.press();

                // only the left button drags
                if button::translate(*button) != Some(Button::Left) {
                    return false;
                }

                pointer.start_drag()
            }

            PointerEventKind::Release { button, .. } => {
                // side buttons like back and forward have no amane button yet
                let Some(button) = button::translate(*button) else {
                    return false;
                };

                if button == Button::Left {
                    pointer.stop_drag();
                }

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
