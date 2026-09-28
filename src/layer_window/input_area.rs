// a part of the window that takes pointer input, from its top left corner
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
