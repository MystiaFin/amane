use crate::Point;

use super::Pointer;

impl Pointer {
    // tells the innermost target under the pointer where the pointer is on it
    pub fn report_motion(&self) -> bool {
        let Some(index) = self.find_topmost(|target| target.handlers.motion.is_some()) else {
            return false;
        };

        let target = &self.targets[index];

        let Some(motion) = &target.handlers.motion else {
            return false;
        };

        let point = Point {
            x: self.x - target.area.x,
            y: self.y - target.area.y,
        };

        motion(point);

        true
    }
}
