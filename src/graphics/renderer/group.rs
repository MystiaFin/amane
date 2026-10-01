use crate::graphics::Renderer;

use super::Command;

impl Renderer {
    pub fn group(&self) -> Self {
        Self {
            commands: Vec::new(),
            transform: self.transform,
        }
    }

    pub fn blend(&mut self, group: Renderer, opacity: f32) {
        let cuts = group
            .commands
            .iter()
            .any(|command| matches!(command, Command::Cut { .. }));

        // a plain group draws the same inline, and only a separate group costs the gpu extra
        if opacity == 1.0 && !cuts {
            self.commands.extend(group.commands);

            return;
        }

        self.commands.push(Command::Group {
            commands: group.commands,
            opacity,
        });
    }
}
