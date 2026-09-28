use crate::graphics::{Rect, Renderer};

use super::Command;

impl Renderer {
    // what the group drew only shows inside the rounded rectangle
    pub fn clip(&mut self, group: Renderer, rect: Rect, radius: f32) {
        // an empty rectangle shows none of the group
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Clip {
            path,
            transform: self.transform,
            commands: group.commands,
        });
    }
}
