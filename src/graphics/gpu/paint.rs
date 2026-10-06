use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Fill, ImageData, Mix};
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;
use crate::graphics::{Area, Corners, Transform};

use super::convert::bezier;
use super::quads::{Clip, Clips};
use super::Gpu;

/*
 * rectangles, borders and letters go to the quads, anything else to vello;
 * whichever was gathering is drawn first when the other takes over, so
 * everything still lands on the canvas in order
 */
impl Gpu {
    // draws each command over what the commands before it left on the canvas
    pub(super) fn run(&mut self, commands: Vec<Command>, canvas: &Texture) {
        self.run_clipped(commands, canvas, Clips::default());
    }

    fn run_clipped(&mut self, commands: Vec<Command>, canvas: &Texture, clips: Clips) {
        let mut scene = Scene::new();

        // blurred copies vello reads from, dropped once the scene using them is drawn
        let mut borrowed = Vec::new();

        self.gather(commands, canvas, clips, &mut scene, &mut borrowed);

        self.flush(&mut scene, canvas, &mut borrowed);
    }

    fn gather(
        &mut self,
        commands: Vec<Command>,
        canvas: &Texture,
        clips: Clips,
        scene: &mut Scene,
        borrowed: &mut Vec<ImageData>,
    ) {
        for command in commands {
            match route(&command) {
                Route::Canvas => {
                    self.flush(scene, canvas, borrowed);

                    self.change_canvas(command, canvas, scene, borrowed);
                }

                Route::OwnCanvas => {
                    self.flush(scene, canvas, borrowed);

                    self.draw_on_own_canvas(command, canvas, clips);
                }

                Route::QuadClip(clip) => {
                    let Command::Clip { commands, .. } = command else {
                        unreachable!("only clips are routed as quad clips");
                    };

                    self.gather(commands, canvas, clips.within(clip), scene, borrowed);
                }

                Route::Quads => {
                    self.paint_pending_vello(scene, canvas, borrowed);

                    self.add_to_quads(command, clips);
                }

                Route::Vello => self.add_to_vello(scene, command, canvas, clips),
            }
        }
    }

    // runs a blur, cut or shader on what the canvas holds so far
    fn change_canvas(
        &mut self,
        command: Command,
        canvas: &Texture,
        scene: &mut Scene,
        borrowed: &mut Vec<ImageData>,
    ) {
        match command {
            Command::Blur {
                path,
                transform,
                amount,
            } => {
                let blurred = self.blur(canvas, scene, &path, transform, amount);

                borrowed.extend(blurred);
            }

            Command::Cut {
                path,
                transform,
                strength,
            } => self.cut(canvas, &path, transform, strength),

            Command::Shader {
                shader,
                values,
                area,
                path,
                transform,
            } => self.shade(canvas, &shader, &values, area, path.as_ref(), transform),

            _ => unreachable!("only blurs, cuts and shaders change the canvas"),
        }
    }

    /*
     * a faded group is drawn on its own canvas and laid down faded as one, so
     * where its parts overlap they don't show through each other; a clip holding
     * a blur, cut or shader is drawn apart too, so those only see the clip's drawing
     */
    fn draw_on_own_canvas(&mut self, command: Command, canvas: &Texture, clips: Clips) {
        let (commands, opacity, clipped_to) = match command {
            Command::Group { commands, opacity } => (commands, opacity, None),

            Command::Clip {
                area,
                radius,
                transform,
                commands,
            } => (commands, 1.0, Some((area, radius, transform))),

            _ => unreachable!("only groups and clips are drawn on their own canvas"),
        };

        let own = self.take_canvas(canvas.width(), canvas.height());

        self.run_clipped(commands, &own, clips);

        if let Some((area, radius, transform)) = clipped_to
            && let Some(path) = area.trace(radius)
        {
            self.trim(&own, &path, transform);
        }

        self.lay(&own, canvas, opacity, false);

        self.give_back(own);
    }

    // route only sends commands here that move and scale without turning
    fn add_to_quads(&mut self, command: Command, clips: Clips) {
        match command {
            Command::Rectangle {
                area,
                radius,
                transform,
                color,
            } => {
                let scale = transform.even_scale().unwrap_or(1.0);

                let area = device_area(area, transform);

                self.quads.rectangle(area, radius.map(|corner| corner * scale), color, clips);
            }

            Command::Border {
                area,
                radius,
                thickness,
                transform,
                color,
            } => {
                let scale = transform.even_scale().unwrap_or(1.0);

                let area = device_area(area, transform);

                self.quads
                    .border(area, radius.map(|corner| corner * scale), thickness * scale, color, clips);
            }

            Command::Glyph {
                face,
                id,
                size,
                color,
                x,
                y,
            } => self
                .quads
                .letter(&self.queue, face, id, size, color, x, y, clips),

            Command::Image {
                image,
                transform,
                area,
                radius,
                clip_transform,
                ..
            } => {
                self.note_shown(&image);

                // the image's own pixels, stretched by its transform onto the canvas
                let size = Area::new(0.0, 0.0, image.width() as f32, image.height() as f32);

                let placement = device_area(size, transform);

                let shape = device_clip(area, radius, clip_transform);

                let clips = match shape {
                    Some(shape) => clips.within(shape),
                    None => clips,
                };

                self.quads
                    .picture(&self.device, &self.queue, &image, placement, clips);
            }

            _ => unreachable!("only rectangles, borders, letters and images go to the quads"),
        }
    }

    // vello's drawing so far goes down first, so the quads land on top of it
    fn paint_pending_vello(
        &mut self,
        scene: &mut Scene,
        canvas: &Texture,
        borrowed: &mut Vec<ImageData>,
    ) {
        if !scene.encoding().is_empty() {
            self.paint(scene, canvas, borrowed);
        }
    }

    // and the other way round, with the quads' clip handed to vello
    fn add_to_vello(
        &mut self,
        scene: &mut Scene,
        command: Command,
        canvas: &Texture,
        clips: Clips,
    ) {
        self.quads.draw(&self.device, &self.queue, canvas);

        let mut paths = Vec::new();

        for clip in [clips.outer, clips.inner].into_iter().flatten() {
            // an empty clip shows nothing of what is inside it
            let Some(path) = clip.area.trace(clip.radius) else {
                return;
            };

            paths.push(path);
        }

        for path in &paths {
            scene.push_layer(
                Fill::NonZero,
                Mix::Normal,
                1.0,
                Affine::IDENTITY,
                &bezier(path),
            );
        }

        self.add(scene, command, canvas);

        for _ in &paths {
            scene.pop_layer();
        }
    }

    fn flush(&mut self, scene: &mut Scene, canvas: &Texture, borrowed: &mut Vec<ImageData>) {
        self.quads.draw(&self.device, &self.queue, canvas);

        self.paint(scene, canvas, borrowed);
    }
}

// where a command is drawn, decided before anything is drawn
enum Route {
    // blurs, cuts and shaders work on what the canvas already holds
    Canvas,

    // a faded group, or a clip holding a blur, cut or shader
    OwnCanvas,

    // a clip the quads apply themselves, its commands routed like any others
    QuadClip(Clip),

    // rectangles, borders, letters and images that move and scale without turning
    Quads,

    // paths, gradients, shadows, and anything turned or stretched
    Vello,
}

fn route(command: &Command) -> Route {
    match command {
        Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => Route::Canvas,

        Command::Group { .. } => Route::OwnCanvas,

        Command::Clip { commands, .. } if needs_own_canvas(commands) => Route::OwnCanvas,

        Command::Clip {
            area,
            radius,
            transform,
            ..
        } => match device_clip(*area, *radius, *transform) {
            Some(clip) => Route::QuadClip(clip),

            // a rotated clip is left to vello, with everything inside it
            None => Route::Vello,
        },

        Command::Rectangle { transform, .. } | Command::Border { transform, .. } => {
            if transform.even_scale().is_some() {
                Route::Quads
            } else {
                Route::Vello
            }
        }

        Command::Glyph { .. } => Route::Quads,

        Command::Image {
            transform,
            clip_transform,
            ..
        } => {
            if straight(*transform) && clip_transform.even_scale().is_some() {
                Route::Quads
            } else {
                Route::Vello
            }
        }

        Command::Fill { .. }
        | Command::Gradient { .. }
        | Command::Stroke { .. }
        | Command::Shadow { .. }
        | Command::InnerShadow { .. } => Route::Vello,
    }
}

// blurs, cuts and shaders work on the canvas itself, so a clip holding one draws on a canvas of its own
fn needs_own_canvas(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => true,
        Command::Group { commands, .. } | Command::Clip { commands, .. } => needs_own_canvas(commands),
        _ => false,
    })
}

// moves and scales without turning or flipping, each side may scale differently
fn straight(transform: Transform) -> bool {
    let turned = transform.kx != 0.0 || transform.ky != 0.0;
    let flipped = transform.sx <= 0.0 || transform.sy <= 0.0;

    !turned && !flipped
}

fn device_area(area: Area, transform: Transform) -> Area {
    Area::new(
        area.x * transform.sx + transform.tx,
        area.y * transform.sy + transform.ty,
        area.width * transform.sx,
        area.height * transform.sy,
    )
}

fn device_clip(area: Area, radius: Corners, transform: Transform) -> Option<Clip> {
    let scale = transform.even_scale()?;

    Some(Clip {
        area: device_area(area, transform),
        radius: radius.map(|corner| corner * scale),
    })
}
