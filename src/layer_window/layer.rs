#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    // under everything
    Background,

    // under normal windows
    Bottom,

    // above normal windows
    Top,

    // above everything
    #[default]
    Overlay,
}
