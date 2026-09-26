use tiny_skia::{FillRule, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};
use ttf_parser::Face;

use super::{Color, Outline, Rect};

pub struct Renderer {
    pixmap: Pixmap,
    transform: Transform,
}

impl Renderer {
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        let pixmap = Pixmap::new(width, height).expect("failed to create pixmap");

        // widgets measure in logical pixels, the pixmap in real ones
        let horizontal_scale = scale;
        let vertical_scale = scale;

        let transform = Transform::from_scale(horizontal_scale, vertical_scale);

        Self { pixmap, transform }
    }

    pub fn clear(&mut self, color: Color) {
        self.pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
    }

    pub fn rectangle(
        &mut self,
        rect: Rect,
        color: Color,
        radius: f32,
        border_thickness: f32,
        border_color: Color,
    ) {
        let Some(path) = trace(rect, radius) else {
            return;
        };

        let mut paint = Paint::default();

        paint.set_color_rgba8(color.r, color.g, color.b, color.a);

        self.pixmap
            .fill_path(&path, &paint, FillRule::Winding, self.transform, None);

        // a zero width line would still draw as a hairline
        if border_thickness == 0.0 {
            return;
        }

        /*
         * a line is drawn centered on its path,
         * so pulling the path in by half the thickness keeps the line inside the edge
         */
        let half_thickness = border_thickness / 2.0;

        let border_rect = Rect::new(
            rect.x + half_thickness,
            rect.y + half_thickness,
            rect.width - border_thickness,
            rect.height - border_thickness,
        );

        // the pulled in path curves tighter, but never past a square corner
        let border_radius = f32::max(radius - half_thickness, 0.0);

        let Some(border_path) = trace(border_rect, border_radius) else {
            return;
        };

        let mut border_paint = Paint::default();

        border_paint.set_color_rgba8(
            border_color.r,
            border_color.g,
            border_color.b,
            border_color.a,
        );

        let stroke = Stroke {
            width: border_thickness,
            ..Stroke::default()
        };

        self.pixmap
            .stroke_path(&border_path, &border_paint, &stroke, self.transform, None);
    }

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

    pub fn into_argb8888(self) -> Vec<u8> {
        let mut pixels = self.pixmap.take();

        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        pixels
    }
}

fn trace(rect: Rect, radius: f32) -> Option<Path> {
    // a radius past half a side would make the corners overlap
    let shortest_half = f32::min(rect.width, rect.height) / 2.0;
    let radius = f32::min(radius, shortest_half);

    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;

    // 0.5523 is how far a curve's handles reach to bend it into a quarter circle
    let handle = radius * (1.0 - 0.5523);

    let mut path = PathBuilder::new();

    path.move_to(left + radius, top);
    path.line_to(right - radius, top);
    path.cubic_to(
        right - handle,
        top,
        right,
        top + handle,
        right,
        top + radius,
    );
    path.line_to(right, bottom - radius);
    path.cubic_to(
        right,
        bottom - handle,
        right - handle,
        bottom,
        right - radius,
        bottom,
    );
    path.line_to(left + radius, bottom);
    path.cubic_to(
        left + handle,
        bottom,
        left,
        bottom - handle,
        left,
        bottom - radius,
    );
    path.line_to(left, top + radius);
    path.cubic_to(left, top + handle, left + handle, top, left + radius, top);
    path.close();

    path.finish()
}
