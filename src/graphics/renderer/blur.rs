use crate::graphics::{Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn blur(&mut self, rect: Rect, radius: f32, amount: f32) {
        // most rectangles ask for no blur, so they skip copying pixels
        if amount == 0.0 {
            return;
        }

        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Blur {
            path,
            transform: self.transform,
            amount,
        });
    }
}
