use crate::graphics::{Color, Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn rectangle(&mut self, rect: Rect, color: Color, radius: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Fill {
            path,
            transform: self.transform,
            color,
        });
    }
}
