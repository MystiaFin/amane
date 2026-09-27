use crate::graphics::{Color, Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn shadow(&mut self, rect: Rect, radius: f32, color: Color, blur: f32) {
        self.commands.push(Command::Shadow {
            rect,
            radius,
            transform: self.transform,
            color,
            blur,
        });
    }

    // the shadow fills the rectangle except for a soft hole, so only a rim is left
    pub fn inner_shadow(&mut self, rect: Rect, hole: Rect, radius: f32, color: Color, blur: f32) {
        let Some(clip) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::InnerShadow {
            clip,
            hole,
            radius,
            transform: self.transform,
            color,
            blur,
        });
    }
}
