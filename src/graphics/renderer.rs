mod command;
mod effect;
mod shape;
mod text;

use super::{Area, Corners, Transform};

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
    pub fn clip(&mut self, group: Renderer, area: Area, radius: Corners) {
        // an empty rectangle shows none of the group
        if area.width <= 0.0 || area.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Clip {
            area,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleFactor;
    use crate::{Color, Parent, Rectangle};

    #[test]
    fn frames_combine_global_and_output_scales_in_drawing() {
        let root = Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Color::BLUE);
        let frame = crate::frame::build(&root, 300, 150, 1.25, ScaleFactor::new(1.5));
        let commands = frame.renderer.finish();
        let Command::Rectangle {
            area, transform, ..
        } = commands[0]
        else {
            panic!("expected a rectangle");
        };

        assert_eq!(area, Area::new(0.0, 0.0, 200.0, 100.0));
        assert_eq!(transform.map(area.width, area.height), (375.0, 187.5));
    }
}
