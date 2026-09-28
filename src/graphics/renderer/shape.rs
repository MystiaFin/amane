use crate::graphics::{Cap, Color, Path, Rect, Renderer, Transform};

use super::Command;

impl Renderer {
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
