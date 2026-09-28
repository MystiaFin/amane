use crate::graphics::{Cap, Color, Rect, Renderer};

use super::Command;

impl Renderer {
    pub fn border(&mut self, rect: Rect, radius: f32, thickness: f32, color: Color) {
        // a zero width line would still draw as a hairline
        if thickness == 0.0 {
            return;
        }

        /*
         * a line is drawn centered on its path,
         * so pulling the path in by half the thickness keeps the line inside the edge
         */
        let half_thickness = thickness / 2.0;

        let border_rect = Rect::new(
            rect.x + half_thickness,
            rect.y + half_thickness,
            rect.width - thickness,
            rect.height - thickness,
        );

        // the pulled in path curves tighter, but never past a square corner
        let border_radius = f32::max(radius - half_thickness, 0.0);

        let Some(border_path) = border_rect.trace(border_radius) else {
            return;
        };

        self.commands.push(Command::Stroke {
            path: border_path,
            transform: self.transform,
            thickness,
            color,
            cap: Cap::Butt,
        });
    }
}

