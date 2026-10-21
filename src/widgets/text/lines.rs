use swash::FontRef;
use swash::zeno::Bounds;

use crate::graphics::font;

// how text that is too wide for its area gets handled
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Rules {
    pub wrap: bool,
    pub elide: bool,
    pub max_lines: Option<usize>,
}

pub fn measure(content: &str, font: FontRef<'static>, size: f32) -> f32 {
    let mut total = 0.0;

    for letter in content.chars() {
        total += font::measure_letter(font, letter, size).advance;
    }

    total
}

// the height of one line, from the top of the tallest letter to the bottom of the lowest
pub fn height(font: FontRef<'static>, size: f32) -> f32 {
    let metrics = font.metrics(font::coords(font)).scale(size);

    // swash counts descent downward from the baseline, so both add up
    metrics.ascent + metrics.descent
}

// the distance from the top of one line to the top of the next
pub fn spacing(font: FontRef<'static>, size: f32) -> f32 {
    let gap = font.metrics(font::coords(font)).scale(size).leading;

    height(font, size) + gap
}

// the height of several lines stacked, without a gap after the last one
pub fn stack_height(font: FontRef<'static>, size: f32, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }

    let above_last = spacing(font, size) * (count - 1) as f32;

    above_last + height(font, size)
}

pub fn arrange(content: &str, font: FontRef<'static>, size: f32, width: f32, rules: Rules) -> Vec<String> {
    let mut lines = match rules.wrap {
        true => wrap(content, font, size, width),
        false => vec![String::from(content)],
    };

    let limit = rules.max_lines.unwrap_or(usize::MAX);

    let cut = lines.len() > limit;

    lines.truncate(limit);

    if !rules.elide {
        return lines;
    }

    let Some(last) = lines.last_mut() else {
        return lines;
    };

    // a line that was cut off also gets the "…", even if it fits
    if cut || measure(last, font, size) > width {
        *last = elide(last, font, size, width);
    }

    lines
}

// fills each line with whole words until the next one would not fit
fn wrap(content: &str, font: FontRef<'static>, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in content.split('\n') {
        let mut line = String::new();

        for word in paragraph.split(' ') {
            // a word wider than the area still gets a line of its own
            if line.is_empty() {
                line = String::from(word);

                continue;
            }

            let longer = format!("{line} {word}");

            if measure(&longer, font, size) <= width {
                line = longer;

                continue;
            }

            lines.push(line);

            line = String::from(word);
        }

        lines.push(line);
    }

    lines
}

// drops letters from the end until the rest and "…" fit
fn elide(line: &str, font: FontRef<'static>, size: f32, width: f32) -> String {
    let mut kept = String::from(line.trim_end());

    loop {
        let shortened = format!("{kept}…");

        if kept.is_empty() || measure(&shortened, font, size) <= width {
            return shortened;
        }

        kept.pop();
    }
}

// where the letters' ink starts and ends, counted from the pen's starting point
pub fn ink(content: &str, font: FontRef<'static>, size: f32) -> (f32, f32) {
    let mut pen = 0.0;

    let mut left = f32::MAX;
    let mut right = f32::MIN;

    for letter in content.chars() {
        let measured = font::measure_letter(font, letter, size);

        // a space has no ink, only an advance
        if let Some(bounds) = bounds(measured.font, measured.id) {
            left = left.min(pen + bounds.min.x * measured.units);
            right = right.max(pen + bounds.max.x * measured.units);
        }

        pen += measured.advance;
    }

    // nothing but spaces is as wide as its advance
    if left > right {
        return (0.0, pen);
    }

    (left, right)
}

// the box around a letter's outline, in the font's own units; none for a space
fn bounds(font: FontRef<'static>, id: u16) -> Option<Bounds> {
    font::with_context(|context| {
        let mut scaler = context
            .builder(font)
            .normalized_coords(font::coords(font))
            .build();

        let shape = scaler.scale_outline(id)?;

        let bounds = shape.bounds();

        if bounds.width() == 0.0 {
            return None;
        }

        Some(bounds)
    })
}
