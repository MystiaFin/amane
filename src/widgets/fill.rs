use crate::Color;

use super::{Image, Mask};

pub enum Fill {
    Color(Color),

    Image(Image),

    // paints nothing and cuts its shape out of the rectangle holding it
    Mask,
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

// a string is a hex color like "#1e1e2e"
impl From<&str> for Fill {
    fn from(hex: &str) -> Self {
        Self::Color(Color::from(hex))
    }
}

impl From<super::Image> for Fill {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

impl From<Mask> for Fill {
    fn from(_: Mask) -> Self {
        Self::Mask
    }
}
