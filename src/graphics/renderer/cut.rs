use crate::graphics::{Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn cut(&mut self, rect: Rect, radius: f32, strength: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Cut {
            path,
            transform: self.transform,
            strength,
        });
    }
}
