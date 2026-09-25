#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub const BLACK: Self = Self::rgb(0, 0, 0);

    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const RED: Self = Self::rgb(255, 0, 0);

    pub const GREEN: Self = Self::rgb(0, 255, 0);

    pub const BLUE: Self = Self::rgb(0, 0, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
