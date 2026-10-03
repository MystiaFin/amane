mod command;
mod effect;
mod shape;
mod text;

use super::{Rect, Transform};

pub use command::Command;
pub use effect::VALUE_ROWS;

/*
 * collects what widgets draw as commands,
 * so no part of amane outside the gpu backend knows how drawing is done
 */
pub struct Renderer {
    commands: Vec<Command>,
    transform: Transform,
}

impl Renderer {
    pub fn new(scale: f32) -> Self {
        // widgets measure in logical pixels, the buffer in real ones
        let horizontal_scale = scale;
        let vertical_scale = scale;

        let transform = Transform::from_scale(horizontal_scale, vertical_scale);

        Self {
            commands: Vec::new(),
            transform,
        }
    }

    pub fn finish(self) -> Vec<Command> {
        self.commands
    }

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

    // what the group drew only shows inside the rounded rectangle
    pub fn clip(&mut self, group: Renderer, rect: Rect, radius: f32) {
        // an empty rectangle shows none of the group
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Clip {
            rect,
            radius,
            transform: self.transform,
            commands: group.commands,
        });
    }

    // everything drawn inside goes through the local transform before the renderer's own
    pub fn transformed(&mut self, local: Transform, draw: impl FnOnce(&mut Renderer)) {
        let outer = self.transform;

        self.transform = local.post_concat(outer);

        draw(self);

        self.transform = outer;
    }
}
