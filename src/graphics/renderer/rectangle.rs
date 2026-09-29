use crate::graphics::{Color, Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn rectangle(&mut self, rect: Rect, color: Color, radius: f32) {
        /*
         * an invisible fill still counts as drawing, and one before a shader
         * makes the gpu run a whole extra drawing pass each frame
         */
        if color.a == 0 {
            return;
        }

        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Rectangle {
            rect,
            radius,
            transform: self.transform,
            color,
        });
    }
}
