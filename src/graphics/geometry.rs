use tiny_skia::{Path, PathBuilder};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn trace(self, radius: f32) -> Option<Path> {
        // a radius past half a side would make the corners overlap
        let shortest_half = f32::min(self.width, self.height) / 2.0;
        let radius = f32::min(radius, shortest_half);

        let left = self.x;
        let top = self.y;
        let right = self.x + self.width;
        let bottom = self.y + self.height;

        // 0.5523 is how far a curve's handles reach to bend it into a quarter circle
        let handle = radius * (1.0 - 0.5523);

        let mut path = PathBuilder::new();

        path.move_to(left + radius, top);
        path.line_to(right - radius, top);
        path.cubic_to(
            right - handle,
            top,
            right,
            top + handle,
            right,
            top + radius,
        );
        path.line_to(right, bottom - radius);
        path.cubic_to(
            right,
            bottom - handle,
            right - handle,
            bottom,
            right - radius,
            bottom,
        );
        path.line_to(left + radius, bottom);
        path.cubic_to(
            left + handle,
            bottom,
            left,
            bottom - handle,
            left,
            bottom - radius,
        );
        path.line_to(left, top + radius);
        path.cubic_to(left, top + handle, left + handle, top, left + radius, top);
        path.close();

        path.finish()
    }
}
