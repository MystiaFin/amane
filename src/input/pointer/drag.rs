use crate::Point;

use super::Pointer;

impl Pointer {
    pub fn start_drag(&mut self) -> bool {
        self.dragged = self.find_topmost(|target| target.handlers.drag.is_some());

        self.drag()
    }

    pub fn stop_drag(&mut self) {
        self.dragged = None;
    }

    // keeps reporting to the target pressed on, even after the pointer leaves it
    pub fn drag(&self) -> bool {
        let Some(index) = self.dragged else {
            return false;
        };

        // a redraw since the press may have removed this target
        let Some(target) = self.targets.get(index) else {
            return false;
        };

        let Some(drag) = &target.handlers.drag else {
            return false;
        };

        let point = Point {
            x: self.x - target.area.x,
            y: self.y - target.area.y,
        };

        drag(point);

        true
    }
}
