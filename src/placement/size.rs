#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Parent,
    Fixed(f32),
}

impl Size {
    pub(crate) fn resolve(self, available: f32) -> f32 {
        match self {
            Size::Parent => available,
            Size::Fixed(pixels) => pixels,
        }
    }
}

impl From<f32> for Size {
    fn from(pixels: f32) -> Self {
        Self::Fixed(pixels)
    }
}
