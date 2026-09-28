pub mod names;

// the pointer's look while it is over a widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    Move,
    NotAllowed,
    Wait,
    Crosshair,

    ResizeTop,
    ResizeBottom,
    ResizeLeft,
    ResizeRight,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
    ResizeHorizontal,
    ResizeVertical,
}
