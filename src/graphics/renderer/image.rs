use std::sync::Arc;

use crate::graphics::image::Bitmap;
use crate::graphics::svg::Svg;
use crate::graphics::{Rect, Renderer, Transform};

use super::Command;

impl Renderer {
    pub fn image(&mut self, rect: Rect, radius: f32, image: Arc<Bitmap>, placement: Rect) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        let horizontal_scale = placement.width / image.width() as f32;
        let vertical_scale = placement.height / image.height() as f32;

        let image_transform = Transform::from_row(
            horizontal_scale,
            0.0,
            0.0,
            vertical_scale,
            placement.x,
            placement.y,
        );

        let transform = image_transform.post_concat(self.transform);

        // the image can reach past the rectangle, the clip keeps it inside the rounded shape
        self.commands.push(Command::Image {
            image,
            transform,
            clip: path,
            rect,
            radius,
            clip_transform: self.transform,
        });
    }
}

impl Renderer {
    // the same for a vector picture, which is scaled instead of stretched pixels
    pub fn svg(&mut self, rect: Rect, radius: f32, svg: Arc<Svg>, placement: Rect) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        let horizontal_scale = placement.width / svg.width();
        let vertical_scale = placement.height / svg.height();

        let svg_transform = Transform::from_row(
            horizontal_scale,
            0.0,
            0.0,
            vertical_scale,
            placement.x,
            placement.y,
        );

        let transform = svg_transform.post_concat(self.transform);

        self.commands.push(Command::Svg {
            svg,
            transform,
            clip: path,
            clip_transform: self.transform,
        });
    }
}
