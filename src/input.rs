pub mod focus;
mod pointer;
mod target;

pub use pointer::Pointer;
pub use target::{Handlers, KeyHandler, Target, clip};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Button {
    #[default]
    Left,

    Right,

    // pressing down on the scroll wheel
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // a key that types something: a letter, digit or symbol, already shifted
    Character(char),

    Enter,

    Escape,

    Tab,

    Backspace,

    Space,

    Up,

    Down,

    Left,

    Right,

    Home,

    End,

    // a key amane has no name for yet
    Other,
}

// where the pointer is, measured from a widget's top-left corner
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// how many pixels a compositor usually reports for one wheel step
pub const PIXELS_PER_LINE: f32 = 15.0;

// measured in lines, so a mouse wheel and a touchpad move things the same distance
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scroll {
    // positive scrolls right
    pub x: f32,

    // positive scrolls down
    pub y: f32,
}

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

pub mod cursor_names {
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
}
