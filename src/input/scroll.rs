// measured in lines, so a mouse wheel and a touchpad move things the same distance
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scroll {
    // positive scrolls right
    pub x: f32,

    // positive scrolls down
    pub y: f32,
}
