use std::sync::{LazyLock, PoisonError};

use swash::FontRef;

use super::{SharedMap, empty_map};

// swash's number for one opened font
type FontId = u64;

// where a variable font's sliders sit, like its weight
type Sliders = &'static [i16];

// the sliders of each opened font; a font without any isn't listed
static STYLE_SLIDERS: SharedMap<FontId, Sliders> = LazyLock::new(empty_map);

// the upper half of fontconfig's number counts named styles from 1, 0 keeps the font's default
pub fn remember(font: FontRef<'static>, style_number: u32) {
    if style_number == 0 {
        return;
    }

    let Some(style) = font.instances().nth(style_number as usize - 1) else {
        return;
    };

    let sliders: Vec<i16> = style.normalized_coords().collect();

    let mut style_sliders = STYLE_SLIDERS.lock().unwrap_or_else(PoisonError::into_inner);

    style_sliders.insert(font.key.value(), Vec::leak(sliders));
}

/*
 * where a variable font's sliders sit, like its weight;
 * empty means the font's default, which is all a plain font has
 */
pub fn coords(font: FontRef<'static>) -> &'static [i16] {
    let style_sliders = STYLE_SLIDERS.lock().unwrap_or_else(PoisonError::into_inner);

    style_sliders.get(&font.key.value()).copied().unwrap_or(&[])
}
