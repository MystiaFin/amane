// a screen the compositor shows windows on, handed to views made once per monitor
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub name: String,

    pub width: u32,
    pub height: u32,
}
