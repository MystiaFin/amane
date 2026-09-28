use smithay_client_toolkit::seat::pointer::CursorIcon;

use crate::Cursor;

use super::WaylandState;

impl WaylandState {
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
