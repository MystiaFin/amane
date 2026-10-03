use vello::Scene;
use vello::wgpu::Texture;

use crate::graphics::renderer::Command;

use super::{Gpu, gradient, shadow, shape};

impl Gpu {
    // hands each command to the file that knows how to draw it
    pub(super) fn add(&mut self, scene: &mut Scene, command: Command, canvas: &Texture) {
        match command {
            Command::Rectangle {
                area,
                radius,
                transform,
                color,
            } => shape::rectangle(scene, area, radius, transform, color),

            Command::Border {
                area,
                radius,
                thickness,
                transform,
                color,
            } => shape::border(scene, area, radius, thickness, transform, color),

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
                area,
                transform,
                gradient,
            } => gradient::fill(scene, &path, area, transform, &gradient),

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
                area,
                radius,
                clip_transform,
            } => self.draw_image(scene, image, transform, area, radius, clip_transform),

            Command::Shadow {
                area,
                radius,
                transform,
                color,
                blur,
            } => shadow::drop_shadow(scene, area, radius, transform, color, blur),

            Command::InnerShadow {
                clip,
                hole,
                radius,
                transform,
                color,
                blur,
            } => shadow::inner_shadow(scene, &clip, hole, radius, transform, color, blur),

            Command::Group { commands, opacity } => self.group(scene, commands, opacity, canvas),

            Command::Clip {
                area,
                radius,
                transform,
                commands,
            } => self.clip(scene, area, radius, transform, commands, canvas),

            // blurs, cuts and shaders split the drawing, so run handles them before they get here
            Command::Blur { .. } | Command::Cut { .. } | Command::Shader { .. } => {}
        }
    }
}
