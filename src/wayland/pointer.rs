use smithay_client_toolkit::seat::pointer::{
    AxisScroll, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, CursorIcon, PointerEvent, PointerEventKind,
    PointerHandler,
};
use wayland_client::{Connection, QueueHandle, protocol::wl_pointer::WlPointer};

use crate::input::PIXELS_PER_LINE;
use crate::{Button, Cursor, Scroll};

use super::WaylandState;

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer,
        events: &[PointerEvent],
    ) {
        let mut handled_on = Vec::new();

        for event in events {
            if self.handle_pointer(event) {
                handled_on.push(event.surface.clone());
            }
        }

        /*
         * a service a handler changed already wakes the windows that read it;
         * only the window the pointer is on redraws for its own state, like a
         * scroll offset, so hovering never redraws every window
         */
        for surface in handled_on {
            self.request_frame_on(&surface);
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
                if to_button(*button) != Some(Button::Left) {
                    return false;
                }

                pointer.start_drag()
            }

            PointerEventKind::Release { button, .. } => {
                // side buttons like back and forward have no amane button yet
                let Some(button) = to_button(*button) else {
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
            } => pointer.scroll(to_scroll(horizontal, vertical)),
        }
    }

    /*
     * uses cursor-shape when the compositor has it,
     * otherwise the pointer draws the icon from the cursor theme
     */
    pub fn show_cursor(&mut self, cursor: Cursor) {
        if self.cursor_shown == Some(cursor) {
            return;
        }

        let Some(pointer) = &self.pointer_device else {
            return;
        };

        // a missing icon in the theme leaves the last cursor in place
        if pointer.set_cursor(&self.connection, icon(cursor)).is_err() {
            return;
        }

        self.cursor_shown = Some(cursor);
    }

    // the compositor puts its own cursor back once the pointer leaves
    pub fn forget_cursor(&mut self) {
        self.cursor_shown = None;
    }
}

fn to_button(code: u32) -> Option<Button> {
    match code {
        BTN_LEFT => Some(Button::Left),
        BTN_RIGHT => Some(Button::Right),
        BTN_MIDDLE => Some(Button::Middle),
        _ => None,
    }
}

fn to_scroll(horizontal: &AxisScroll, vertical: &AxisScroll) -> Scroll {
    Scroll {
        x: lines(horizontal),
        y: lines(vertical),
    }
}

fn lines(axis: &AxisScroll) -> f32 {
    // a wheel reports its steps in 120ths, which is exact
    if axis.value120 != 0 {
        return axis.value120 as f32 / 120.0;
    }

    // older compositors report whole steps instead
    if axis.discrete != 0 {
        return axis.discrete as f32;
    }

    // a touchpad only reports pixels
    let pixels = axis.absolute as f32;

    pixels / PIXELS_PER_LINE
}

fn icon(cursor: Cursor) -> CursorIcon {
    match cursor {
        Cursor::Default => CursorIcon::Default,
        Cursor::Pointer => CursorIcon::Pointer,
        Cursor::Text => CursorIcon::Text,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::Move => CursorIcon::Move,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::Wait => CursorIcon::Wait,
        Cursor::Crosshair => CursorIcon::Crosshair,

        Cursor::ResizeTop => CursorIcon::NResize,
        Cursor::ResizeBottom => CursorIcon::SResize,
        Cursor::ResizeLeft => CursorIcon::WResize,
        Cursor::ResizeRight => CursorIcon::EResize,
        Cursor::ResizeTopLeft => CursorIcon::NwResize,
        Cursor::ResizeTopRight => CursorIcon::NeResize,
        Cursor::ResizeBottomLeft => CursorIcon::SwResize,
        Cursor::ResizeBottomRight => CursorIcon::SeResize,
        Cursor::ResizeHorizontal => CursorIcon::EwResize,
        Cursor::ResizeVertical => CursorIcon::NsResize,
    }
}
