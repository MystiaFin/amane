use super::PathBuilder;

impl PathBuilder {
    /*
     * angles are in degrees, 0 is straight up and they grow clockwise,
     * a negative sweep goes counterclockwise
     */
    pub fn arc(&mut self, center_x: f32, center_y: f32, radius: f32, start: f32, sweep: f32) {
        let (start_x, start_y) = point_at(center_x, center_y, radius, start);

        // an arc in the middle of a path joins on to it instead of starting a new piece
        if self.segments.is_empty() {
            self.move_to(start_x, start_y);
        } else {
            self.line_to(start_x, start_y);
        }

        // one curve can only follow a circle closely for up to a quarter turn
        let quarters = f32::ceil(f32::abs(sweep) / 90.0);
        let pieces = f32::max(quarters, 1.0);

        let piece_sweep = sweep / pieces;

        for piece in 0..pieces as u32 {
            let from = start + piece_sweep * piece as f32;

            self.arc_piece(center_x, center_y, radius, from, piece_sweep);
        }
    }

    fn arc_piece(&mut self, center_x: f32, center_y: f32, radius: f32, from: f32, sweep: f32) {
        let to = from + sweep;

        let (from_x, from_y) = point_at(center_x, center_y, radius, from);
        let (to_x, to_y) = point_at(center_x, center_y, radius, to);

        // how far the handles reach along the circle's direction to bend the curve onto it
        let handle = radius * 4.0 / 3.0 * f32::tan(f32::to_radians(sweep) / 4.0);

        let (from_direction_x, from_direction_y) = direction_at(from);
        let (to_direction_x, to_direction_y) = direction_at(to);

        self.cubic_to(
            from_x + handle * from_direction_x,
            from_y + handle * from_direction_y,
            to_x - handle * to_direction_x,
            to_y - handle * to_direction_y,
            to_x,
            to_y,
        );
    }
}

fn point_at(center_x: f32, center_y: f32, radius: f32, angle: f32) -> (f32, f32) {
    let radians = f32::to_radians(angle);

    // y grows downward on screen, so up is minus
    let x = center_x + radius * f32::sin(radians);
    let y = center_y - radius * f32::cos(radians);

    (x, y)
}

// which way a point moving clockwise around the circle heads at this angle
fn direction_at(angle: f32) -> (f32, f32) {
    let radians = f32::to_radians(angle);

    (f32::cos(radians), f32::sin(radians))
}
