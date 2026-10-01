mod kind;

use crate::Color;
use crate::graphics::Rect;

pub use kind::Kind;

pub struct Shadow {
    pub(crate) color: Color,
    pub(crate) opacity: f32,
    pub(crate) blur: f32,

    pub(crate) x: f32,
    pub(crate) y: f32,

    pub(crate) kind: Kind,
}

impl Shadow {
    pub fn drop(color: impl Into<Color>) -> Self {
        Self::new(color.into(), Kind::Drop)
    }

    pub fn inner(color: impl Into<Color>) -> Self {
        Self::new(color.into(), Kind::Inner)
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;

        self
    }

    pub fn blur(mut self, amount: f32) -> Self {
        self.blur = amount;

        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;

        self
    }

    fn new(color: Color, kind: Kind) -> Self {
        Self {
            color,
            opacity: 1.0,
            blur: 0.0,
            x: 0.0,
            y: 0.0,
            kind,
        }
    }

    // opacity thins out whatever alpha the color already has
    pub(crate) fn tint(&self) -> Color {
        let opacity = f32::clamp(self.opacity, 0.0, 1.0);

        let alpha = f32::from(self.color.a) * opacity;

        Color::rgba(
            self.color.r,
            self.color.g,
            self.color.b,
            alpha.round() as u8,
        )
    }

    pub(crate) fn place(&self, area: Rect) -> Rect {
        Rect::new(area.x + self.x, area.y + self.y, area.width, area.height)
    }
}
