use crate::graphics::Rect;

// empty space kept between a rectangle's edges and its child
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    // the area left once the padding is taken off every side
    pub(crate) fn shrink(self, area: Rect) -> Rect {
        let width = f32::max(area.width - self.left - self.right, 0.0);
        let height = f32::max(area.height - self.top - self.bottom, 0.0);

        Rect::new(area.x + self.left, area.y + self.top, width, height)
    }
}

// the same space on every side
impl From<f32> for Padding {
    fn from(pixels: f32) -> Self {
        Self {
            top: pixels,
            right: pixels,
            bottom: pixels,
            left: pixels,
        }
    }
}
