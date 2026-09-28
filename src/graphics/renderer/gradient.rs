use crate::graphics::{Gradient, Rect, Renderer};

use super::Command;

impl Renderer {
    // the gradient spreads across the rectangle, not the whole window
    pub fn gradient(&mut self, rect: Rect, gradient: &Gradient, radius: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Gradient {
            path,
            rect,
            transform: self.transform,
            gradient: gradient.clone(),
        });
    }
}
