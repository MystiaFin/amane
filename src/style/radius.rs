use crate::Full;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radius {
    Full,
    Fixed(f32),
}

impl Radius {
    pub(crate) fn resolve(self, width: f32, height: f32) -> f32 {
        match self {
            Radius::Full => f32::min(width, height) / 2.0,
            Radius::Fixed(pixels) => pixels,
        }
    }
}

impl From<f32> for Radius {
    fn from(pixels: f32) -> Self {
        Self::Fixed(pixels)
    }
}

impl From<Full> for Radius {
    fn from(_: Full) -> Self {
        Self::Full
    }
}
