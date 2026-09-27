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
        let cuts = layer
            .commands
            .iter()
            .any(|command| matches!(command, Command::Cut { .. }));

        // a plain group draws the same inline, and only a separate layer costs the gpu extra
        if opacity == 1.0 && !cuts {
            self.commands.extend(layer.commands);

            return;
        }

        self.commands.push(Command::Layer {
            commands: layer.commands,
            opacity,
        });
    }
}
