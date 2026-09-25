#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn distance(&self, x: f32, y: f32, radius: f32) -> f32 {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        let center_x = self.x + half_width;
        let center_y = self.y + half_height;

        // a radius past half a side would make the corners overlap
        let shortest_half = f32::min(half_width, half_height);
        let radius = f32::min(radius, shortest_half);

        // the shape is symmetric, so mirror every point into one corner
        let folded_x = (x - center_x).abs();
        let folded_y = (y - center_y).abs();

        // how far the point sits past the rect shrunk by the radius
        let past_x = folded_x - (half_width - radius);
        let past_y = folded_y - (half_height - radius);

        // past the shrunk rect: straight-line distance to it
        let outside_x = f32::max(past_x, 0.0);
        let outside_y = f32::max(past_y, 0.0);
        let outside = f32::hypot(outside_x, outside_y);

        // within the shrunk rect: distance to its nearest edge, as a negative
        let nearest_edge = f32::max(past_x, past_y);
        let inside = f32::min(nearest_edge, 0.0);

        // growing the shrunk rect back out by the radius rounds the corners
        outside + inside - radius
    }
}
