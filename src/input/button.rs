#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Button {
    #[default]
    Left,

    Right,

    // pressing down on the scroll wheel
    Middle,
}
