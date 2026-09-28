mod arc;

// one step of a path, in the path's own coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    MoveTo(f32, f32),

    LineTo(f32, f32),

    QuadTo(f32, f32, f32, f32),

    CubicTo(f32, f32, f32, f32, f32, f32),

    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub(crate) segments: Vec<Segment>,
}

#[derive(Debug, Clone, Default)]
pub struct PathBuilder {
    segments: Vec<Segment>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.segments.push(Segment::MoveTo(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(Segment::LineTo(x, y));
    }

    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.segments.push(Segment::QuadTo(x1, y1, x, y));
    }

    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.segments.push(Segment::CubicTo(x1, y1, x2, y2, x, y));
    }

    pub fn close(&mut self) {
        self.segments.push(Segment::Close);
    }

    // a path with no steps draws nothing, so there is no path to hand back
    pub fn finish(self) -> Option<Path> {
        if self.segments.is_empty() {
            return None;
        }

        Some(Path {
            segments: self.segments,
        })
    }
}
