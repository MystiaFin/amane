#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Keyboard {
    // never takes keyboard focus
    #[default]
    None,

    // takes all keyboard input while open
    Exclusive,

    // takes focus when clicked
    OnDemand,
}
