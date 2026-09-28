use ttf_parser::Face;

// how text that is too wide for its area gets handled
#[derive(Debug, Clone, Copy, Default)]
pub struct Rules {
    pub wrap: bool,
    pub elide: bool,
    pub max_lines: Option<usize>,
}

pub fn measure(content: &str, font: &Face, size: f32) -> f32 {
    // fonts measure in their own units, this turns them into pixels
    let units = size / f32::from(font.units_per_em());

    let mut total = 0.0;

    for letter in content.chars() {
        // a letter the font lacks draws as its placeholder box
        let id = font.glyph_index(letter).unwrap_or_default();

        let advance = font
            .glyph_hor_advance(id)
            .expect("failed to read letter advance");

        total += f32::from(advance) * units;
    }

    total
}

// the height of one line, from the top of the tallest letter to the bottom of the lowest
pub fn height(font: &Face, size: f32) -> f32 {
    let units = size / f32::from(font.units_per_em());

    let ascent = f32::from(font.ascender()) * units;
    let descent = f32::from(font.descender()) * units;

    // descent is negative, so this adds the part below the baseline
    ascent - descent
}

// the distance from the top of one line to the top of the next
pub fn spacing(font: &Face, size: f32) -> f32 {
    let units = size / f32::from(font.units_per_em());

    let gap = f32::from(font.line_gap()) * units;

    height(font, size) + gap
}

// the height of several lines stacked, without a gap after the last one
pub fn stack_height(font: &Face, size: f32, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }

    let above_last = spacing(font, size) * (count - 1) as f32;

    above_last + height(font, size)
}

pub fn arrange(content: &str, font: &Face, size: f32, width: f32, rules: Rules) -> Vec<String> {
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
fn wrap(content: &str, font: &Face, size: f32, width: f32) -> Vec<String> {
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
fn elide(line: &str, font: &Face, size: f32, width: f32) -> String {
    let mut kept = String::from(line.trim_end());

    loop {
        let shortened = format!("{kept}…");

        if kept.is_empty() || measure(&shortened, font, size) <= width {
            return shortened;
        }

        kept.pop();
    }
}
