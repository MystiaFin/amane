use tiny_skia::{FillRule, Paint, Transform};
use ttf_parser::Face;

use crate::graphics::{Color, Outline, Rect, Renderer};

impl Renderer {
    pub fn text(&mut self, content: &str, font: &Face, size: f32, color: Color, area: Rect) {
        // fonts measure in their own units, this turns them into pixels
        let units = size / f32::from(font.units_per_em());

        let baseline = area.y + f32::from(font.ascender()) * units;

        let mut pen_x = area.x;

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        for letter in content.chars() {
            // a letter the font lacks draws as its placeholder box
            let id = font.glyph_index(letter).unwrap_or_default();

            let advance = font
                .glyph_hor_advance(id)
                .expect("failed to read letter advance");

            let letter_x = pen_x;

            pen_x += f32::from(advance) * units;

            let mut outline = Outline::new();

            // a space has no outline but still moves the pen
            if font.outline_glyph(id, &mut outline).is_none() {
                continue;
            }

            let Some(path) = outline.finish() else {
                continue;
            };

            // fonts point y upward, the pixmap points it downward
            let letter_transform = Transform::from_row(units, 0.0, 0.0, -units, letter_x, baseline);
            let transform = letter_transform.post_concat(self.transform);

            self.pixmap
                .fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
    }
}
