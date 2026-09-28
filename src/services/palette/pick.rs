use crate::Color;

// wcag's minimum contrast for normal sized text
const READABLE: f32 = 4.5;

// how much a vivid color counts over one near middle brightness, taken from jaqc
const SATURATION_WEIGHT: f32 = 1.4;
const TARGET_LUMINANCE: f32 = 0.52;
const LUMINANCE_PENALTY: f32 = 0.35;

pub fn darkest(colors: &[Color]) -> Option<Color> {
    let mut darkest = *colors.first()?;

    for &color in colors {
        if luminance(color) < luminance(darkest) {
            darkest = color;
        }
    }

    Some(darkest)
}

// saturated but not too dark or too bright, the color that stands out
pub fn most_vivid(colors: &[Color]) -> Option<Color> {
    let mut vivid = *colors.first()?;
    let mut vivid_score = f32::MIN;

    for &color in colors {
        let saturation = saturation(color) * SATURATION_WEIGHT;
        let distance = (luminance(color) - TARGET_LUMINANCE).abs() * LUMINANCE_PENALTY;

        let score = saturation - distance;

        if score > vivid_score {
            vivid = color;
            vivid_score = score;
        }
    }

    Some(vivid)
}

/*
 * the palette color with the most contrast keeps the image's tint,
 * but only if it's readable, otherwise plain white or black
 */
pub fn readable_on(background: Color, colors: &[Color]) -> Color {
    let mut best = background;
    let mut best_contrast = 1.0;

    for &color in colors {
        let color_contrast = contrast(color, background);

        if color_contrast > best_contrast {
            best = color;
            best_contrast = color_contrast;
        }
    }

    if best_contrast < READABLE {
        return plain_on(background);
    }

    best
}

// white or black, whichever stands out more
pub fn plain_on(background: Color) -> Color {
    if contrast(Color::WHITE, background) >= contrast(Color::BLACK, background) {
        return Color::WHITE;
    }

    Color::BLACK
}

// 1 for the same brightness, up to 21 for black on white
pub fn contrast(first: Color, second: Color) -> f32 {
    let first = luminance(first);
    let second = luminance(second);

    let lighter = first.max(second);
    let darker = first.min(second);

    (lighter + 0.05) / (darker + 0.05)
}

// how bright the eye sees it, 0 to 1
pub fn luminance(color: Color) -> f32 {
    let red = linear(color.r) * 0.2126;
    let green = linear(color.g) * 0.7152;
    let blue = linear(color.b) * 0.0722;

    red + green + blue
}

// screens store brightness on a curve, this undoes it
fn linear(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;

    if value <= 0.04045 {
        return value / 12.92;
    }

    ((value + 0.055) / 1.055).powf(2.4)
}

fn saturation(color: Color) -> f32 {
    let highest = color.r.max(color.g).max(color.b);
    let lowest = color.r.min(color.g).min(color.b);

    f32::from(highest - lowest) / 255.0
}
