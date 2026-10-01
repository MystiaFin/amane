use crate::graphics::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    // fills the rectangle exactly, bending the image's shape to match
    Stretch,

    // fills the whole rectangle, cutting off what spills past the edges
    Cover,

    // shows the whole image, leaving the rest of the rectangle uncovered
    Contain,
}

impl Fit {
    pub fn place(self, area: Rect, image_width: f32, image_height: f32) -> Rect {
        let horizontal_scale = area.width / image_width;
        let vertical_scale = area.height / image_height;

        let scale = match self {
            Fit::Stretch => return area,
            Fit::Cover => f32::max(horizontal_scale, vertical_scale),
            Fit::Contain => f32::min(horizontal_scale, vertical_scale),
        };

        let width = image_width * scale;
        let height = image_height * scale;

        let x = area.x + (area.width - width) / 2.0;
        let y = area.y + (area.height - height) / 2.0;

        Rect::new(x, y, width, height)
    }
}
