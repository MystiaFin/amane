use crate::graphics::Renderer;

use super::Command;

impl Renderer {
    pub fn layer(&self) -> Self {
        Self {
            commands: Vec::new(),
            transform: self.transform,
        }
    }

    pub fn blend(&mut self, layer: Renderer, opacity: f32) {
        self.commands.push(Command::Layer {
            commands: layer.commands,
            opacity,
        });
    }
}
