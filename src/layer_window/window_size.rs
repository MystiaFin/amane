#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowSize {
    Full,
    Fixed(f32),
}

impl From<f32> for WindowSize {
    fn from(pixels: f32) -> Self {
        Self::Fixed(pixels)
    }
}
