use vello::Scene;
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;

use super::{Gpu, texture};

impl Gpu {
    // draws each command over what the commands before it left on the canvas
    pub(super) fn run(&mut self, commands: Vec<Command>, canvas: &Texture) {
        let mut scene = Scene::new();

        // blurred copies vello reads from, dropped once the scene using them is drawn
        let mut borrowed = Vec::new();

        for command in commands {
            match command {
                Command::Blur {
                    path,
                    transform,
                    amount,
                } => {
                    self.paint(&mut scene, canvas, &mut borrowed);

                    let blurred = self.blur(canvas, &mut scene, &path, transform, amount);

                    borrowed.extend(blurred);
                }

                Command::Cut {
                    path,
                    transform,
                    strength,
                } => {
                    self.paint(&mut scene, canvas, &mut borrowed);

                    self.cut(canvas, &path, transform, strength);
                }

                /*
                 * a blur inside a layer has to see the layer's own drawing,
                 * and a cut inside one must stop at its edge, so the layer gets its own canvas
                 */
                Command::Layer { commands, opacity } if separate(&commands) => {
                    self.paint(&mut scene, canvas, &mut borrowed);

                    let layer = texture::canvas(&self.device, canvas.width(), canvas.height());

                    self.run(commands, &layer);

                    self.lay(&layer, canvas, opacity, false);
                }

                command => self.add(&mut scene, command, canvas),
            }
        }

        self.paint(&mut scene, canvas, &mut borrowed);
    }
}

fn separate(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } | Command::Cut { .. } => true,
        Command::Layer { commands, .. } => separate(commands),
        _ => false,
    })
}
