use std::path::Path;

use crate::graphics::{Rect, Renderer};

use super::Command;

impl Renderer {
    // runs the shader file over the rounded rectangle
    pub fn shader(&mut self, rect: Rect, shader: &Path, radius: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Shader {
            shader: shader.to_path_buf(),
            rect,
            path,
            transform: self.transform,
        });
    }
}
