use vello::Scene;
use vello::kurbo::{self, Affine};
use vello::peniko::{Fill, Mix};
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;

use super::Gpu;

impl Gpu {
    // the commands are drawn into a group that is then faded as one
    pub(super) fn layer(
        &mut self,
        scene: &mut Scene,
        commands: Vec<Command>,
        opacity: f32,
        canvas: &Texture,
    ) {
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
}
