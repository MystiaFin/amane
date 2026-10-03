use std::path::Path;

use crate::graphics::{Color, Rect, Renderer};

use super::Command;

// how many rows of four numbers a shader can be handed as values
pub const VALUE_ROWS: usize = 16;

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

    // runs the shader file over the rounded rectangle, handing it the values
    pub fn shader(&mut self, rect: Rect, shader: &Path, radius: f32, values: &[[f32; 4]]) {
        let Some(outline) = rect.trace(radius) else {
            return;
        };

        // the shader only covers the rect, so square corners need no trimming
        let path = if radius > 0.0 { Some(outline) } else { None };

        self.commands.push(Command::Shader {
            shader: shader.to_path_buf(),
            values: values.to_vec(),
            rect,
            path,
            transform: self.transform,
        });
    }
}
