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

                Command::Shader {
                    shader,
                    values,
                    rect,
                    path,
                    transform,
                } => {
                    self.paint(&mut scene, canvas, &mut borrowed);

                    self.shade(canvas, &shader, &values, rect, path.as_ref(), transform);
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

                // the same for a clip, whose canvas is trimmed to the path before it is laid down
                Command::Clip {
                    path,
                    transform,
                    commands,
                } if separate(&commands) => {
                    self.paint(&mut scene, canvas, &mut borrowed);

                    let layer = texture::canvas(&self.device, canvas.width(), canvas.height());

                    self.run(commands, &layer);

                    self.trim(&layer, &path, transform);

                    self.lay(&layer, canvas, 1.0, false);
                }

                command => self.add(&mut scene, command, canvas),
            }
        }

        self.paint(&mut scene, canvas, &mut borrowed);
    }
}

fn separate(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => true,
        Command::Layer { commands, .. } | Command::Clip { commands, .. } => separate(commands),
        _ => false,
    })
}
