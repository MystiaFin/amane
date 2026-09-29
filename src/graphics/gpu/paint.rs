use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Fill, ImageData, Mix};
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;
use crate::graphics::{Rect, Transform};

use super::convert::bezier;
use super::quads::Clip;
use super::{Gpu, texture};

/*
 * rectangles, borders and letters go to the quads, anything else to vello;
 * whichever was gathering is drawn first when the other takes over, so
 * everything still lands on the canvas in order
 */
impl Gpu {
    // draws each command over what the commands before it left on the canvas
    pub(super) fn run(&mut self, commands: Vec<Command>, canvas: &Texture) {
        self.run_clipped(commands, canvas, None);
    }

    fn run_clipped(&mut self, commands: Vec<Command>, canvas: &Texture, clip: Option<Clip>) {
        let mut scene = Scene::new();

        // blurred copies vello reads from, dropped once the scene using them is drawn
        let mut borrowed = Vec::new();

        self.gather(commands, canvas, clip, &mut scene, &mut borrowed);

        self.flush(&mut scene, canvas, &mut borrowed);
    }

    fn gather(
        &mut self,
        commands: Vec<Command>,
        canvas: &Texture,
        clip: Option<Clip>,
        scene: &mut Scene,
        borrowed: &mut Vec<ImageData>,
    ) {
        for command in commands {
            match command {
                Command::Blur {
                    path,
                    transform,
                    amount,
                } => {
                    self.flush(scene, canvas, borrowed);

                    let blurred = self.blur(canvas, scene, &path, transform, amount);

                    borrowed.extend(blurred);
                }

                Command::Cut {
                    path,
                    transform,
                    strength,
                } => {
                    self.flush(scene, canvas, borrowed);

                    self.cut(canvas, &path, transform, strength);
                }

                Command::Shader {
                    shader,
                    values,
                    rect,
                    path,
                    transform,
                } => {
                    self.flush(scene, canvas, borrowed);

                    self.shade(canvas, &shader, &values, rect, path.as_ref(), transform);
                }

                /*
                 * a faded group is drawn on its own canvas and laid down faded as one,
                 * so where its parts overlap they don't show through each other
                 */
                Command::Layer { commands, opacity } => {
                    self.flush(scene, canvas, borrowed);

                    let layer = texture::canvas(&self.device, canvas.width(), canvas.height());

                    self.run_clipped(commands, &layer, clip);

                    self.lay(&layer, canvas, opacity, false);
                }

                // a blur inside a clip has to see only the clip's own drawing, so it gets a canvas
                Command::Clip {
                    path,
                    transform,
                    commands,
                    ..
                } if separate(&commands) => {
                    self.flush(scene, canvas, borrowed);

                    let layer = texture::canvas(&self.device, canvas.width(), canvas.height());

                    self.run_clipped(commands, &layer, clip);

                    self.trim(&layer, &path, transform);

                    self.lay(&layer, canvas, 1.0, false);
                }

                Command::Clip {
                    path,
                    rect,
                    radius,
                    transform,
                    commands,
                } => {
                    let Some(inner) = device_clip(rect, radius, transform) else {
                        // a rotated clip is left to vello, with everything inside it
                        let command = Command::Clip {
                            path,
                            rect,
                            radius,
                            transform,
                            commands,
                        };

                        self.to_vello(scene, command, canvas, clip);

                        continue;
                    };

                    let clip = Some(narrow(clip, inner));

                    self.gather(commands, canvas, clip, scene, borrowed);
                }

                Command::Rectangle {
                    rect,
                    radius,
                    transform,
                    color,
                } if even_scale(transform).is_some() => {
                    self.to_quads(scene, canvas, borrowed);

                    let scale = even_scale(transform).unwrap_or(1.0);

                    let rect = device_rect(rect, transform);

                    self.quads.rectangle(rect, radius * scale, color, clip);
                }

                Command::Border {
                    rect,
                    radius,
                    thickness,
                    transform,
                    color,
                } if even_scale(transform).is_some() => {
                    self.to_quads(scene, canvas, borrowed);

                    let scale = even_scale(transform).unwrap_or(1.0);

                    let rect = device_rect(rect, transform);

                    self.quads
                        .border(rect, radius * scale, thickness * scale, color, clip);
                }

                Command::Glyph {
                    face,
                    id,
                    size,
                    color,
                    x,
                    y,
                } => {
                    self.to_quads(scene, canvas, borrowed);

                    self.quads
                        .letter(&self.queue, face, id, size, color, x, y, clip);
                }

                Command::Image {
                    image,
                    transform,
                    rect,
                    radius,
                    clip_transform,
                    ..
                } if straight(transform) && even_scale(clip_transform).is_some() => {
                    self.to_quads(scene, canvas, borrowed);

                    // the image's own pixels, stretched by its transform onto the canvas
                    let size = Rect::new(0.0, 0.0, image.width() as f32, image.height() as f32);

                    let placement = device_rect(size, transform);

                    let shape = device_clip(rect, radius, clip_transform);

                    let clip = shape.map(|shape| narrow(clip, shape));

                    self.quads
                        .picture(&self.device, &self.queue, image, placement, clip);
                }

                command => self.to_vello(scene, command, canvas, clip),
            }
        }
    }

    // vello's drawing so far goes down first, so the quads land on top of it
    fn to_quads(&mut self, scene: &mut Scene, canvas: &Texture, borrowed: &mut Vec<ImageData>) {
        if !scene.encoding().is_empty() {
            self.paint(scene, canvas, borrowed);
        }
    }

    // and the other way round, with the quads' clip handed to vello
    fn to_vello(
        &mut self,
        scene: &mut Scene,
        command: Command,
        canvas: &Texture,
        clip: Option<Clip>,
    ) {
        self.quads.draw(&self.device, &self.queue, canvas);

        let Some(clip) = clip else {
            self.add(scene, command, canvas);

            return;
        };

        let Some(path) = clip.rect.trace(clip.radius) else {
            return;
        };

        scene.push_layer(
            Fill::NonZero,
            Mix::Normal,
            1.0,
            Affine::IDENTITY,
            &bezier(&path),
        );

        self.add(scene, command, canvas);

        scene.pop_layer();
    }

    fn flush(&mut self, scene: &mut Scene, canvas: &Texture, borrowed: &mut Vec<ImageData>) {
        self.quads.draw(&self.device, &self.queue, canvas);

        self.paint(scene, canvas, borrowed);
    }
}

fn separate(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => true,
        Command::Layer { commands, .. } | Command::Clip { commands, .. } => separate(commands),
        _ => false,
    })
}

// the scale when the transform only moves and scales evenly, none when it rotates or stretches
fn even_scale(transform: Transform) -> Option<f32> {
    let even = (transform.sx - transform.sy).abs() < 0.0001;
    let straight = transform.kx == 0.0 && transform.ky == 0.0;

    if !even || !straight || transform.sx <= 0.0 {
        return None;
    }

    Some(transform.sx)
}

// moves and scales without turning or flipping, each side may scale differently
fn straight(transform: Transform) -> bool {
    let turned = transform.kx != 0.0 || transform.ky != 0.0;
    let flipped = transform.sx <= 0.0 || transform.sy <= 0.0;

    !turned && !flipped
}

fn device_rect(rect: Rect, transform: Transform) -> Rect {
    Rect::new(
        rect.x * transform.sx + transform.tx,
        rect.y * transform.sy + transform.ty,
        rect.width * transform.sx,
        rect.height * transform.sy,
    )
}

fn device_clip(rect: Rect, radius: f32, transform: Transform) -> Option<Clip> {
    let scale = even_scale(transform)?;

    Some(Clip {
        rect: device_rect(rect, transform),
        radius: radius * scale,
    })
}

/*
 * only one rounded clip reaches the quads, so nested clips become their
 * overlap, rounded like the inner one; clips inside clips are almost
 * always smaller, so this is exact for them
 */
fn narrow(outer: Option<Clip>, inner: Clip) -> Clip {
    let Some(outer) = outer else {
        return inner;
    };

    let left = f32::max(outer.rect.x, inner.rect.x);
    let top = f32::max(outer.rect.y, inner.rect.y);

    let outer_right = outer.rect.x + outer.rect.width;
    let outer_bottom = outer.rect.y + outer.rect.height;

    let right = f32::min(outer_right, inner.rect.x + inner.rect.width);
    let bottom = f32::min(outer_bottom, inner.rect.y + inner.rect.height);

    let width = f32::max(right - left, 0.0);
    let height = f32::max(bottom - top, 0.0);

    Clip {
        rect: Rect::new(left, top, width, height),
        radius: inner.radius,
    }
}
