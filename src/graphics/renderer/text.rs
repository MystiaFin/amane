use std::cell::RefCell;
use std::collections::HashMap;

use ttf_parser::{Face, GlyphId};

use crate::graphics::{Color, Outline, Path, Rect, Renderer, Transform, font};

use super::Command;

thread_local! {
    /*
     * each letter's outline, read from the font once; fonts stay loaded
     * for the whole run, so where a face lives in memory names it
     */
    static OUTLINES: RefCell<HashMap<(usize, u16), Option<Path>>> = RefCell::new(HashMap::new());
}

impl Renderer {
    /*
     * letters are drawn as small cached pictures, which the gpu handles far
     * faster than outlines; only rotated or stretched text needs the outlines
     */
    pub fn text(
        &mut self,
        content: &str,
        font: &'static Face<'static>,
        size: f32,
        color: Color,
        area: Rect,
    ) {
        match self.transform.even_scale() {
            Some(scale) => self.text_pictures(content, font, size * scale, color, area),
            None => self.text_outlines(content, font, size, color, area),
        }
    }

    /*
     * the line starts on a whole pixel and every letter keeps its rounded
     * distance from that start, so a moving line never shimmers apart
     */
    fn text_pictures(
        &mut self,
        content: &str,
        font: &'static Face<'static>,
        pixel_size: f32,
        color: Color,
        area: Rect,
    ) {
        let units = pixel_size / f32::from(font.units_per_em());

        let start = device_point(self.transform, area.x, area.y);

        let line_x = start.0.round();
        let baseline = (start.1 + f32::from(font.ascender()) * units).round();

        let mut pen = 0.0;

        for letter in content.chars() {
            let (face, id) = font::with_letter(font, letter);

            let advance = face
                .glyph_hor_advance(id)
                .expect("failed to read letter advance");

            // a fallback font can measure in other units than the text's own
            let face_units = pixel_size / f32::from(face.units_per_em());

            let letter_x = line_x + f32::round(pen);

            pen += f32::from(advance) * face_units;

            self.commands.push(Command::Glyph {
                face,
                id: id.0,
                size: pixel_size,
                color,
                x: letter_x,
                y: baseline,
            });
        }
    }

    fn text_outlines(&mut self, content: &str, font: &Face, size: f32, color: Color, area: Rect) {
        // fonts measure in their own units, this turns them into pixels
        let units = size / f32::from(font.units_per_em());

        let baseline = area.y + f32::from(font.ascender()) * units;

        let mut pen_x = area.x;

        for letter in content.chars() {
            let (face, id) = font::with_letter(font, letter);

            let advance = face
                .glyph_hor_advance(id)
                .expect("failed to read letter advance");

            // a fallback font can measure in other units than the text's own
            let face_units = size / f32::from(face.units_per_em());

            let letter_x = pen_x;

            pen_x += f32::from(advance) * face_units;

            // a space has no outline but still moves the pen
            let Some(path) = outline(face, id) else {
                continue;
            };

            // fonts point y upward, the screen points it downward
            let letter_transform =
                Transform::from_row(face_units, 0.0, 0.0, -face_units, letter_x, baseline);
            let transform = letter_transform.post_concat(self.transform);

            self.commands.push(Command::Fill {
                path,
                transform,
                color,
            });
        }
    }
}

fn device_point(transform: Transform, x: f32, y: f32) -> (f32, f32) {
    (
        transform.sx * x + transform.kx * y + transform.tx,
        transform.ky * x + transform.sy * y + transform.ty,
    )
}

fn outline(font: &Face, id: GlyphId) -> Option<Path> {
    let key = (std::ptr::from_ref(font) as usize, id.0);

    OUTLINES.with_borrow_mut(|outlines| {
        let path = outlines
            .entry(key)
            .or_insert_with(|| read_outline(font, id));

        path.clone()
    })
}

fn read_outline(font: &Face, id: GlyphId) -> Option<Path> {
    let mut outline = Outline::new();

    font.outline_glyph(id, &mut outline)?;

    outline.finish()
}
