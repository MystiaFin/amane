use std::sync::Arc;

use crate::graphics::image::Bitmap;
use crate::graphics::{Rect, Renderer, Transform};

use super::Command;

impl Renderer {
    pub fn image(&mut self, rect: Rect, radius: f32, image: Arc<Bitmap>, placement: Rect) {
        // an empty rectangle shows none of the image
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

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
            rect,
            radius,
            clip_transform: self.transform,
        });
    }
}
