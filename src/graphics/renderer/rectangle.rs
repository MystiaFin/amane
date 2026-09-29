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
