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

    // a key amane has no name for yet
    Other,
}
