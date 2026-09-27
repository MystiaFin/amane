use vello::peniko::{self, ImageData};
use vello::wgpu::Texture;
use vello::{AaConfig, RenderParams, Scene};

use crate::graphics::renderer::Command;

use super::{Gpu, shadow, shape, texture};

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

                // a layer with a blur inside has to see its own drawing, so it gets its own canvas
                Command::Layer { commands, opacity } if blurs(&commands) => {
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

    // hands each command to the file that knows how to draw it
    pub(super) fn add(&mut self, scene: &mut Scene, command: Command, canvas: &Texture) {
        match command {
            Command::Fill {
                path,
                transform,
                color,
            } => shape::fill(scene, &path, transform, color),

            Command::Stroke {
                path,
                transform,
                thickness,
                color,
            } => shape::stroke(scene, &path, transform, thickness, color),

            Command::Image {
                image,
                transform,
                clip,
                clip_transform,
            } => self.draw_image(scene, image, transform, &clip, clip_transform),

            Command::Shadow {
                rect,
                radius,
                transform,
                color,
                blur,
            } => shadow::drop_shadow(scene, rect, radius, transform, color, blur),

            Command::InnerShadow {
                clip,
                hole,
                radius,
                transform,
                color,
                blur,
            } => shadow::inner_shadow(scene, &clip, hole, radius, transform, color, blur),

            Command::Layer { commands, opacity } => self.layer(scene, commands, opacity, canvas),

            // blurs split the drawing, so run handles them before they get here
            Command::Blur { .. } => {}
        }
    }

    // vello always clears what it draws into, so it draws apart and the result is laid on top
    fn paint(&mut self, scene: &mut Scene, canvas: &Texture, borrowed: &mut Vec<ImageData>) {
        if scene.encoding().is_empty() {
            return;
        }

        self.keep_atlas(scene);

        let scratch = texture::scratch(&self.device, canvas.width(), canvas.height());

        let params = RenderParams {
            base_color: peniko::Color::TRANSPARENT,
            width: canvas.width(),
            height: canvas.height(),
            antialiasing_method: AaConfig::Area,
        };

        self.vello
            .render_to_texture(
                &self.device,
                &self.queue,
                scene,
                &texture::view(&scratch),
                &params,
            )
            .expect("failed to draw scene");

        self.lay(&scratch, canvas, 1.0, true);

        scene.reset();

        for image in borrowed.drain(..) {
            self.vello.unregister_texture(image);
        }
    }

    pub(super) fn lay(&self, source: &Texture, canvas: &Texture, opacity: f32, premultiply: bool) {
        let premultiply = if premultiply { 1.0 } else { 0.0 };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.composite.run(
            &self.device,
            &mut encoder,
            &texture::view(source),
            &texture::view(canvas),
            [opacity, premultiply, 0.0, 0.0],
        );

        self.queue.submit([encoder.finish()]);
    }
}

fn blurs(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } => true,
        Command::Layer { commands, .. } => blurs(commands),
        _ => false,
    })
}
