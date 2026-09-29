mod blur;
mod border;
mod clip;
mod command;
mod cut;
mod gradient;
mod image;
mod layer;
mod rectangle;
mod shadow;
mod shader;
mod shape;
mod text;
mod transform;

use super::Transform;

pub use command::Command;
pub use shader::VALUE_ROWS;

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
