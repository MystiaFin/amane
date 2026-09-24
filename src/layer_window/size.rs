#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    // stretch across the whole screen
    Full,

    Fixed(u32),
}

// lets users write 30 instead of Size::Fixed(30)
impl From<u32> for Size {
    fn from(pixels: u32) -> Self {
        Self::Fixed(pixels)
    }
}
