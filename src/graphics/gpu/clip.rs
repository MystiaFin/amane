use vello::Scene;
use vello::kurbo::{self, Affine, Shape};
use vello::peniko::{Fill, Mix};
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;
use crate::graphics::{Color, Path, Rect, Transform};

use super::Gpu;
use super::convert::{affine, bezier, paint};

impl Gpu {
    // the commands are drawn into a group that vello only shows inside the rounded rectangle
    pub(super) fn clip(
        &mut self,
        scene: &mut Scene,
        rect: Rect,
        radius: f32,
        transform: Transform,
        commands: Vec<Command>,
        canvas: &Texture,
    ) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        scene.push_layer(
            Fill::NonZero,
            Mix::Normal,
            1.0,
            affine(transform),
            &bezier(&path),
        );

        for command in commands {
            self.add(scene, command, canvas);
        }

        scene.pop_layer();
    }

    // erases everything on the canvas that lies outside the path
    pub(super) fn trim(&mut self, canvas: &Texture, path: &Path, transform: Transform) {
        let window = kurbo::Rect::new(
            0.0,
            0.0,
            f64::from(canvas.width()),
            f64::from(canvas.height()),
        );

        let mut outside = affine(transform) * bezier(path);

        for element in window.path_elements(0.1) {
            outside.push(element);
        }

        // with the window around the path, even-odd fills only the part between them
        let mut scene = Scene::new();

        scene.fill(
            Fill::EvenOdd,
            Affine::IDENTITY,
            paint(Color::WHITE),
            None,
            &outside,
        );

        self.erase_covered(&scene, canvas, 1.0);
    }
}
