use std::sync::Arc;

use crate::graphics::image::Bitmap;
use crate::graphics::{Cap, Color, Gradient, Path, Rect, Renderer, Transform};

use super::Command;

impl Renderer {
    pub fn rectangle(&mut self, rect: Rect, color: Color, radius: f32) {
        /*
         * an invisible fill still counts as drawing, and one before a shader
         * makes the gpu run a whole extra drawing pass each frame
         */
        if color.a == 0 {
            return;
        }

        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Rectangle {
            rect,
            radius,
            transform: self.transform,
            color,
        });
    }

    pub fn border(&mut self, rect: Rect, radius: f32, thickness: f32, color: Color) {
        // a zero width line would still draw as a hairline
        if thickness == 0.0 || color.a == 0 {
            return;
        }

        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Border {
            rect,
            radius,
            thickness,
            transform: self.transform,
            color,
        });
    }

    // the gradient spreads across the rectangle, not the whole window
    pub fn gradient(&mut self, rect: Rect, gradient: &Gradient, radius: f32) {
        let Some(path) = rect.trace(radius) else {
            return;
        };

        self.commands.push(Command::Gradient {
            path,
            rect,
            transform: self.transform,
            gradient: gradient.clone(),
        });
    }

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

    // the path is in the area's own coordinates, with 0,0 at its top left corner
    pub fn fill_path(&mut self, path: &Path, area: Rect, color: Color) {
        self.commands.push(Command::Fill {
            path: path.clone(),
            transform: self.inside(area),
            color,
        });
    }

    pub fn stroke_path(&mut self, path: &Path, area: Rect, thickness: f32, color: Color, cap: Cap) {
        self.commands.push(Command::Stroke {
            path: path.clone(),
            transform: self.inside(area),
            thickness,
            color,
            cap,
        });
    }

    // moves the path to where the area sits before scaling it like everything else
    fn inside(&self, area: Rect) -> Transform {
        let offset = Transform::from_row(1.0, 0.0, 0.0, 1.0, area.x, area.y);

        offset.post_concat(self.transform)
    }
}
