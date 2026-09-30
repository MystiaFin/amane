use vello::peniko::{self, ImageData};
use vello::wgpu::Texture;
use vello::{AaConfig, RenderParams, Scene};

use super::{Gpu, texture};

impl Gpu {
    // vello always clears what it draws into, so it draws apart and the result is laid on top
    pub(super) fn paint(
        &mut self,
        scene: &mut Scene,
        canvas: &Texture,
        borrowed: &mut Vec<ImageData>,
    ) {
        if scene.encoding().is_empty() {
            return;
        }

        // the same place in the frame usually gets the same scene as last frame
        let index = self.painting;

        self.painting += 1;

        if let Some(kept) = self.unchanged(index, scene, canvas) {
            self.lay(kept, canvas, 1.0, true);
        } else {
            let scratch = self.render(scene, canvas);

            self.lay(&scratch, canvas, 1.0, true);

            self.keep(index, scene, scratch);
        }

        scene.reset();

        for image in borrowed.drain(..) {
            self.vello.unregister_texture(image);
        }
    }

    // the scene alone, on its own texture the size of the canvas, given back by the caller
    pub(super) fn render(&mut self, scene: &Scene, canvas: &Texture) -> Texture {
        self.keep_atlas(scene);

        let scratch = self.take_scratch(canvas.width(), canvas.height());

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

        scratch
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
