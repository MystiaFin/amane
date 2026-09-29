use std::path::Path;

use crate::graphics::{Rect, Renderer};

use super::Command;

// how many rows of four numbers a shader can be handed as values
pub const VALUE_ROWS: usize = 16;

impl Renderer {
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
