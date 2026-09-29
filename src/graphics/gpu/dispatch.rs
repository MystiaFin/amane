use vello::Scene;
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;

use super::{Gpu, gradient, shadow, shape};

impl Gpu {
    // hands each command to the file that knows how to draw it
    pub(super) fn add(&mut self, scene: &mut Scene, command: Command, canvas: &Texture) {
        match command {
            Command::Glyph {
                face,
                id,
                size,
                color,
                x,
                y,
            } => self.draw_glyph(scene, face, id, size, color, x, y),

            Command::Fill {
                path,
                transform,
                color,
            } => shape::fill(scene, &path, transform, color),

            Command::Gradient {
                path,
                rect,
                transform,
                gradient,
            } => gradient::fill(scene, &path, rect, transform, &gradient),

            Command::Stroke {
                path,
                transform,
                thickness,
                color,
                cap,
            } => shape::stroke(scene, &path, transform, thickness, color, cap),

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

            Command::Clip {
                path,
                transform,
                commands,
            } => self.clip(scene, &path, transform, commands, canvas),

            // blurs, cuts and shaders split the drawing, so run handles them before they get here
            Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => {}
        }
    }
}
