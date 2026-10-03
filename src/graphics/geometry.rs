use super::{BezierPath, PathBuilder};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Area {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        let right = self.x + self.width;
        let bottom = self.y + self.height;

        let inside_horizontally = x >= self.x && x < right;
        let inside_vertically = y >= self.y && y < bottom;

        inside_horizontally && inside_vertically
    }

    // the part both rectangles cover, empty when they don't meet
    pub fn intersect(self, other: Area) -> Area {
        let left = f32::max(self.x, other.x);
        let top = f32::max(self.y, other.y);
        let right = f32::min(self.x + self.width, other.x + other.width);
        let bottom = f32::min(self.y + self.height, other.y + other.height);

        let width = f32::max(right - left, 0.0);
        let height = f32::max(bottom - top, 0.0);

        Area::new(left, top, width, height)
    }

    pub fn trace(self, radius: f32) -> Option<BezierPath> {
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
