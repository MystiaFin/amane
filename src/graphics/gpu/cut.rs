use vello::Scene;
use vello::wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Device, Texture, TextureFormat,
};

use crate::graphics::{Color, Path, Transform};

use super::pass::Pass;
use super::{Gpu, shape, texture};

impl Gpu {
    // the shape is drawn alone, then as much as it covers is taken away from the canvas
    pub(super) fn cut(
        &mut self,
        canvas: &Texture,
        path: &Path,
        transform: Transform,
        strength: f32,
    ) {
        let mut scene = Scene::new();

        shape::fill(&mut scene, path, transform, Color::WHITE);

        self.erase_covered(&scene, canvas, strength);
    }

    // takes away from the canvas as much as the scene covers
    pub(super) fn erase_covered(&mut self, scene: &Scene, canvas: &Texture, strength: f32) {
        let coverage = self.render(scene, canvas);

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // the shape still needs premultiplying, and its alpha is scaled by the strength
        self.erase.run(
            &self.device,
            &mut encoder,
            &texture::view(&coverage),
            &texture::view(canvas),
            [strength, 1.0, 0.0, 0.0],
        );

        self.queue.submit([encoder.finish()]);

        self.give_back(coverage);
    }
}

// keeps only the part of the canvas the shape leaves uncovered, whatever color the shape is
pub(super) fn erase(device: &Device) -> Pass {
    let keep_uncovered = BlendComponent {
        src_factor: BlendFactor::Zero,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };

    let blend = BlendState {
        color: keep_uncovered,
        alpha: keep_uncovered,
    };

    Pass::new(device, "composite", TextureFormat::Rgba8Unorm, Some(blend))
}
