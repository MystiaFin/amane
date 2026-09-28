#![allow(non_upper_case_globals)]

use super::Cursor;

/*
 * values, not types, so `Text` and `Pointer` can sit next to
 * the Text widget without a Cursor:: prefix
 */
pub const Default: Cursor = Cursor::Default;
pub const Pointer: Cursor = Cursor::Pointer;
pub const Text: Cursor = Cursor::Text;
pub const Grab: Cursor = Cursor::Grab;
pub const Grabbing: Cursor = Cursor::Grabbing;
pub const Move: Cursor = Cursor::Move;
pub const NotAllowed: Cursor = Cursor::NotAllowed;
pub const Wait: Cursor = Cursor::Wait;
pub const Crosshair: Cursor = Cursor::Crosshair;

pub const ResizeTop: Cursor = Cursor::ResizeTop;
pub const ResizeBottom: Cursor = Cursor::ResizeBottom;
pub const ResizeLeft: Cursor = Cursor::ResizeLeft;
pub const ResizeRight: Cursor = Cursor::ResizeRight;
pub const ResizeTopLeft: Cursor = Cursor::ResizeTopLeft;
pub const ResizeTopRight: Cursor = Cursor::ResizeTopRight;
pub const ResizeBottomLeft: Cursor = Cursor::ResizeBottomLeft;
pub const ResizeBottomRight: Cursor = Cursor::ResizeBottomRight;
pub const ResizeHorizontal: Cursor = Cursor::ResizeHorizontal;
pub const ResizeVertical: Cursor = Cursor::ResizeVertical;
