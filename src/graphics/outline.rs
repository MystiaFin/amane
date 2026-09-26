use tiny_skia::{Path, PathBuilder};
use ttf_parser::OutlineBuilder;

pub struct Outline {
    path: PathBuilder,
}

impl Outline {
    pub fn new() -> Self {
        Self {
            path: PathBuilder::new(),
        }
    }

    pub fn finish(self) -> Option<Path> {
        self.path.finish()
    }
}

impl OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.path.close();
    }
}
