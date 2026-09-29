use crate::graphics::{Color, Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn border(&mut self, rect: Rect, radius: f32, thickness: f32, color: Color) {
        // a zero width line would still draw as a hairline
        if thickness == 0.0 || color.a == 0 {
            return;
        }

        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Border {
            rect,
            radius,
            thickness,
            transform: self.transform,
            color,
        });
    }
}
