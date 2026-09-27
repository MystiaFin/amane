use vello::kurbo::{self, Affine, BezPath, Join, Stroke};
use vello::peniko::{
    self, Blob, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality, Mix,
};
use vello::wgpu::Texture;
use vello::{AaConfig, RenderParams, Scene};

use crate::graphics::image::Bitmap;
use crate::graphics::path::Segment;
use crate::graphics::renderer::Command;
use crate::graphics::{Color, Path, Transform};

use super::{Gpu, pass};

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

                    let layer = pass::canvas(&self.device, canvas.width(), canvas.height());

                    self.run(commands, &layer);

                    self.lay(&layer, canvas, opacity, false);
                }

                command => self.add(&mut scene, command, canvas),
            }
        }

        self.paint(&mut scene, canvas, &mut borrowed);
    }

    fn add(&mut self, scene: &mut Scene, command: Command, canvas: &Texture) {
        match command {
            Command::Fill {
                path,
                transform,
                color,
            } => scene.fill(
                Fill::NonZero,
                affine(transform),
                paint(color),
                None,
                &bezier(&path),
            ),

            Command::Stroke {
                path,
                transform,
                thickness,
                color,
            } => {
                // square corners stay square instead of being rounded off by the line
                let stroke = Stroke::new(f64::from(thickness)).with_join(Join::Miter);

                scene.stroke(
                    &stroke,
                    affine(transform),
                    paint(color),
                    None,
                    &bezier(&path),
                );
            }

            Command::Image {
                image,
                transform,
                clip,
                clip_transform,
            } => {
                // bicubic keeps a large image smooth when it shrinks to fit
                let brush = ImageBrush::new(self.image(image)).with_quality(ImageQuality::High);

                scene.push_clip_layer(Fill::NonZero, affine(clip_transform), &bezier(&clip));

                scene.draw_image(&brush, affine(transform));

                scene.pop_layer();
            }

            Command::Layer { commands, opacity } => {
                let window = kurbo::Rect::new(
                    0.0,
                    0.0,
                    f64::from(canvas.width()),
                    f64::from(canvas.height()),
                );

                scene.push_layer(
                    Fill::NonZero,
                    Mix::Normal,
                    opacity,
                    Affine::IDENTITY,
                    &window,
                );

                for command in commands {
                    self.add(scene, command, canvas);
                }

                scene.pop_layer();
            }

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

        let scratch = pass::scratch(&self.device, canvas.width(), canvas.height());

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
                &pass::view(&scratch),
                &params,
            )
            .expect("failed to draw scene");

        self.lay(&scratch, canvas, 1.0, true);

        scene.reset();

        for image in borrowed.drain(..) {
            self.vello.unregister_texture(image);
        }
    }

    /*
     * vello throws its image atlas away after a scene without images,
     * but still counts the images in it as uploaded, so they are sent again
     */
    fn keep_atlas(&mut self, scene: &Scene) {
        if scene.encoding().resources.patches.is_empty() {
            self.atlas_dropped = true;

            return;
        }

        if !self.atlas_dropped {
            return;
        }

        for image in self.images.values() {
            self.vello.mark_override_image_dirty(image);
        }

        self.atlas_dropped = false;
    }

    pub(super) fn lay(&self, source: &Texture, canvas: &Texture, opacity: f32, premultiply: bool) {
        let premultiply = if premultiply { 1.0 } else { 0.0 };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.composite.run(
            &self.device,
            &mut encoder,
            &pass::view(source),
            &pass::view(canvas),
            [opacity, premultiply, 0.0, 0.0],
        );

        self.queue.submit([encoder.finish()]);
    }

    // images live until the program exits, so where one lives says which image it is
    fn image(&mut self, image: &'static Bitmap) -> ImageData {
        let key = std::ptr::from_ref(image) as usize;

        let converted = self.images.entry(key).or_insert_with(|| ImageData {
            data: Blob::from(image.pixels.clone()),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: image.width(),
            height: image.height(),
        });

        converted.clone()
    }
}

fn blurs(commands: &[Command]) -> bool {
    commands.iter().any(|command| match command {
        Command::Blur { .. } => true,
        Command::Layer { commands, .. } => blurs(commands),
        _ => false,
    })
}

pub(super) fn bezier(path: &Path) -> BezPath {
    let mut bezier = BezPath::new();

    for segment in &path.segments {
        match *segment {
            Segment::MoveTo(x, y) => bezier.move_to(point(x, y)),

            Segment::LineTo(x, y) => bezier.line_to(point(x, y)),

            Segment::QuadTo(x1, y1, x, y) => bezier.quad_to(point(x1, y1), point(x, y)),

            Segment::CubicTo(x1, y1, x2, y2, x, y) => {
                bezier.curve_to(point(x1, y1), point(x2, y2), point(x, y))
            }

            Segment::Close => bezier.close_path(),
        }
    }

    bezier
}

pub(super) fn affine(transform: Transform) -> Affine {
    Affine::new([
        f64::from(transform.sx),
        f64::from(transform.ky),
        f64::from(transform.kx),
        f64::from(transform.sy),
        f64::from(transform.tx),
        f64::from(transform.ty),
    ])
}

fn paint(color: Color) -> peniko::Color {
    peniko::Color::from_rgba8(color.r, color.g, color.b, color.a)
}

fn point(x: f32, y: f32) -> (f64, f64) {
    (f64::from(x), f64::from(y))
}
