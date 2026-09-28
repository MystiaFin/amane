mod blur;
mod border;
mod command;
mod cut;
mod image;
mod layer;
mod rectangle;
mod shadow;
mod shape;
mod text;

use super::Transform;

pub use command::Command;

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
}
