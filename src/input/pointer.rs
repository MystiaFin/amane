mod click;
mod cursor;
mod drag;
mod hover;
mod motion;
mod scroll;

use super::Target;

#[derive(Default)]
pub struct Pointer {
    // the hit areas of the last drawn frame
    targets: Vec<Target>,

    x: f32,
    y: f32,

    // positions in targets, which stay the same across redraws of an unchanged layout
    hovered: Vec<usize>,
    pressed: Option<usize>,
    dragged: Option<usize>,
}

impl Pointer {
    // the hit areas of a new frame replace the last frame's
    pub fn set_targets(&mut self, targets: Vec<Target>) {
        self.targets = targets;
    }

    // the innermost target under the pointer that has the handler asked for
    fn find_topmost(&self, has_handler: fn(&Target) -> bool) -> Option<usize> {
        let mut topmost = None;

        // children are collected after their parents, so the last match is the innermost
        for (index, target) in self.targets.iter().enumerate() {
            if target.contains(self.x, self.y) && has_handler(target) {
                topmost = Some(index);
            }
        }

        topmost
    }
}
