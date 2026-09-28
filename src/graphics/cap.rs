// how the two ends of an open line are finished
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cap {
    // the line stops flat at its end point
    #[default]
    Butt,

    // a half circle past the end point
    Round,

    // a half square past the end point
    Square,
}
