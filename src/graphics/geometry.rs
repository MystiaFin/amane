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

    pub fn trace(self, radius: Corners) -> Option<BezierPath> {
        // a radius past half a side would make the corners overlap
        let shortest_half = f32::min(self.width, self.height) / 2.0;

        let Corners {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } = radius.map(|corner| f32::min(corner, shortest_half));

        let left = self.x;
        let top = self.y;
        let right = self.x + self.width;
        let bottom = self.y + self.height;

        // 0.5523 is how far a curve's handles reach to bend it into a quarter circle
        let handle = |corner: f32| corner * (1.0 - 0.5523);

        let mut path = PathBuilder::new();

        path.move_to(left + top_left, top);
        path.line_to(right - top_right, top);
        path.cubic_to(
            right - handle(top_right),
            top,
            right,
            top + handle(top_right),
            right,
            top + top_right,
        );
        path.line_to(right, bottom - bottom_right);
        path.cubic_to(
            right,
            bottom - handle(bottom_right),
            right - handle(bottom_right),
            bottom,
            right - bottom_right,
            bottom,
        );
        path.line_to(left + bottom_left, bottom);
        path.cubic_to(
            left + handle(bottom_left),
            bottom,
            left,
            bottom - handle(bottom_left),
            left,
            bottom - bottom_left,
        );
        path.line_to(left, top + top_left);
        path.cubic_to(
            left,
            top + handle(top_left),
            left + handle(top_left),
            top,
            left + top_left,
            top,
        );
        path.close();

        path.finish()
    }
}

// how far each corner of a rectangle curves
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    pub fn map(self, change: impl Fn(f32) -> f32) -> Self {
        Self {
            top_left: change(self.top_left),
            top_right: change(self.top_right),
            bottom_right: change(self.bottom_right),
            bottom_left: change(self.bottom_left),
        }
    }

    pub fn largest(self) -> f32 {
        self.to_array().into_iter().fold(0.0, f32::max)
    }

    // clockwise from the top left, the order quads.wgsl reads them in
    pub fn to_array(self) -> [f32; 4] {
        [self.top_left, self.top_right, self.bottom_right, self.bottom_left]
    }
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}
