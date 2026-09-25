#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Zone {
    // keeps other windows out of the edge the window sits on
    Reserve,

    // reserves nothing, stays out of what others reserve
    #[default]
    Respect,

    // reserves nothing, covers what others reserve
    Ignore,
}
