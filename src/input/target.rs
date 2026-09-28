use crate::Point;
use crate::graphics::{Rect, Transform};

use super::Handlers;

// where a widget was drawn, and what it does when the pointer uses that spot
pub struct Target {
    pub area: Rect,
    pub handlers: Handlers,

    // turns a window position into the area's own space, undoing rotated, scaled or moved parents
    pub inverse: Transform,
}

impl Target {
    pub fn new(area: Rect, handlers: Handlers) -> Self {
        Self {
            area,
            handlers,
            inverse: Transform::IDENTITY,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        let (local_x, local_y) = self.inverse.map(x, y);

        self.area.contains(local_x, local_y)
    }

    // the position measured from the area's top-left corner
    pub fn point(&self, x: f32, y: f32) -> Point {
        let (local_x, local_y) = self.inverse.map(x, y);

        Point {
            x: local_x - self.area.x,
            y: local_y - self.area.y,
        }
    }
}
