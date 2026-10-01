#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    // cast behind the rectangle, onto whatever is under it
    #[default]
    Drop,

    // along the inside edge, as if the rectangle were pressed in
    Inner,
}
