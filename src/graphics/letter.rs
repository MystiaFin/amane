use ttf_parser::{Face, GlyphId, OutlineBuilder};

// one letter turned into pixels: how much of each pixel it covers, 0 to 255
pub struct Letter {
    pub width: u32,
    pub height: u32,

    // where the pixels sit from the pen on the baseline
    pub left: f32,
    pub top: f32,

    pub coverage: Vec<u8>,
}

/*
 * reads the letter's outline from the font only when it is asked for, so a
 * font with thousands of icons costs nothing until one is drawn; none for a
 * letter without pixels, like a space
 */
pub fn rasterize(font: &Face, id: u16, size: f32) -> Option<Letter> {
    let bounds = font.glyph_bounding_box(GlyphId(id))?;

    let scale = size / f32::from(font.units_per_em());

    // a pixel of room on every side, so no edge falls outside the grid
    let left = (f32::from(bounds.x_min) * scale).floor() - 1.0;
    let top = (-f32::from(bounds.y_max) * scale).floor() - 1.0;

    let right = (f32::from(bounds.x_max) * scale).ceil() + 1.0;
    let bottom = (-f32::from(bounds.y_min) * scale).ceil() + 1.0;

    let width = (right - left) as usize;
    let height = (bottom - top) as usize;

    if width == 0 || height == 0 {
        return None;
    }

    let mut grid = Grid {
        width,
        height,
        scale,
        left,
        top,
        areas: vec![0.0; width * height + 1],
        start: (0.0, 0.0),
        pen: (0.0, 0.0),
    };

    font.outline_glyph(GlyphId(id), &mut grid)?;

    Some(Letter {
        width: width as u32,
        height: height as u32,
        left,
        top,
        coverage: grid.coverage(),
    })
}

/*
 * every edge adds the area it covers to the pixels it crosses, signed by
 * whether it goes up or down; adding the areas up along each row then gives
 * how much of every pixel lies inside the letter
 */
struct Grid {
    width: usize,
    height: usize,

    // font units to pixels, and where the grid's corner is from the pen
    scale: f32,
    left: f32,
    top: f32,

    areas: Vec<f32>,

    // where the current contour started, and where the pen is now, in pixels
    start: (f32, f32),
    pen: (f32, f32),
}

impl Grid {
    // fonts point y upward, the grid points it downward
    fn to_pixels(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.scale - self.left, -y * self.scale - self.top)
    }

    fn coverage(&self) -> Vec<u8> {
        let mut coverage = Vec::with_capacity(self.width * self.height);

        let mut total = 0.0;

        for area in &self.areas[..self.width * self.height] {
            total += area;

            let amount = f32::min(total.abs(), 1.0);

            coverage.push((amount * 255.0).round() as u8);
        }

        coverage
    }

    // the edge from one point to the next, in pixels
    fn edge(&mut self, from: (f32, f32), to: (f32, f32)) {
        if from.1 == to.1 {
            return;
        }

        let (direction, top, bottom) = if from.1 < to.1 {
            (1.0, from, to)
        } else {
            (-1.0, to, from)
        };

        let slope = (bottom.0 - top.0) / (bottom.1 - top.1);

        let first_row = top.1.max(0.0) as usize;
        let last_row = (bottom.1.ceil() as usize).min(self.height);

        let mut x = top.0 + (first_row as f32 - top.1).max(0.0) * slope;

        for row in first_row..last_row {
            let row_top = f32::max(row as f32, top.1);
            let row_bottom = f32::min((row + 1) as f32, bottom.1);

            let height = row_bottom - row_top;

            let next_x = x + slope * height;

            let signed = height * direction;

            self.row(row, x, next_x, signed);

            x = next_x;
        }
    }

    // spreads one row's slice of an edge over the pixels it passes
    fn row(&mut self, row: usize, x: f32, next_x: f32, signed: f32) {
        let start = row * self.width;

        let (low, high) = if x < next_x { (x, next_x) } else { (next_x, x) };

        let low_floor = low.floor();
        let high_ceil = high.ceil();

        let low_index = low_floor as usize;
        let high_index = high_ceil as usize;

        // within one pixel: its share goes to it, the rest to the pixel after
        if high_index <= low_index + 1 {
            let middle = 0.5 * (x + next_x) - low_floor;

            self.areas[start + low_index] += signed - signed * middle;
            self.areas[start + low_index + 1] += signed * middle;

            return;
        }

        let per_pixel = 1.0 / (high - low);

        let low_part = low - low_floor;
        let first = 0.5 * per_pixel * (1.0 - low_part) * (1.0 - low_part);

        let high_part = high - high_ceil + 1.0;
        let last = 0.5 * per_pixel * high_part * high_part;

        self.areas[start + low_index] += signed * first;

        if high_index == low_index + 2 {
            self.areas[start + low_index + 1] += signed * (1.0 - first - last);
        } else {
            let second = per_pixel * (1.5 - low_part);

            self.areas[start + low_index + 1] += signed * (second - first);

            for index in low_index + 2..high_index - 1 {
                self.areas[start + index] += signed * per_pixel;
            }

            let before_last = second + (high_index - low_index - 3) as f32 * per_pixel;

            self.areas[start + high_index - 1] += signed * (1.0 - before_last - last);
        }

        self.areas[start + high_index] += signed * last;
    }

    // curves become enough short lines that no step is longer than about a pixel
    fn steps(points: &[(f32, f32)]) -> usize {
        let mut length = 0.0;

        for pair in points.windows(2) {
            let dx = pair[1].0 - pair[0].0;
            let dy = pair[1].1 - pair[0].1;

            length += (dx * dx + dy * dy).sqrt();
        }

        (length.ceil() as usize).clamp(1, 64)
    }
}

impl OutlineBuilder for Grid {
    fn move_to(&mut self, x: f32, y: f32) {
        let point = self.to_pixels(x, y);

        self.start = point;
        self.pen = point;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let point = self.to_pixels(x, y);

        self.edge(self.pen, point);

        self.pen = point;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = self.to_pixels(x1, y1);
        let end = self.to_pixels(x, y);

        let from = self.pen;

        let steps = Self::steps(&[from, control, end]);

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let rest = 1.0 - t;

            let point = (
                rest * rest * from.0 + 2.0 * rest * t * control.0 + t * t * end.0,
                rest * rest * from.1 + 2.0 * rest * t * control.1 + t * t * end.1,
            );

            self.edge(self.pen, point);

            self.pen = point;
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let first = self.to_pixels(x1, y1);
        let second = self.to_pixels(x2, y2);
        let end = self.to_pixels(x, y);

        let from = self.pen;

        let steps = Self::steps(&[from, first, second, end]);

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let rest = 1.0 - t;

            let a = rest * rest * rest;
            let b = 3.0 * rest * rest * t;
            let c = 3.0 * rest * t * t;
            let d = t * t * t;

            let point = (
                a * from.0 + b * first.0 + c * second.0 + d * end.0,
                a * from.1 + b * first.1 + c * second.1 + d * end.1,
            );

            self.edge(self.pen, point);

            self.pen = point;
        }
    }

    fn close(&mut self) {
        self.edge(self.pen, self.start);

        self.pen = self.start;
    }
}
