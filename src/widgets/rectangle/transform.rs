use crate::Rectangle;
use crate::graphics::{Area, Transform};

impl Rectangle {
    // clockwise, around the rectangle's center
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = degrees;

        self
    }

    // around the rectangle's center, 1 keeps the size
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;

        self
    }

    // moves the drawing and the hit area without changing the layout
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;

        self
    }
}

// scales, then rotates, then moves, all around the center of the area
pub fn local(rectangle: &Rectangle, area: Area) -> Transform {
    let center_x = area.x + area.width / 2.0;
    let center_y = area.y + area.height / 2.0;

    let to_origin = Transform::from_translate(-center_x, -center_y);
    let scale = Transform::from_scale(rectangle.scale, rectangle.scale);
    let rotate = Transform::from_rotate(rectangle.rotation);

    let back_x = center_x + rectangle.translate_x;
    let back_y = center_y + rectangle.translate_y;

    let back = Transform::from_translate(back_x, back_y);

    to_origin
        .post_concat(scale)
        .post_concat(rotate)
        .post_concat(back)
}
