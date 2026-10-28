use std::sync::{LazyLock, PoisonError};

use fontconfig::{CharSet, Fontconfig, Pattern, UnicodeCoverage};
use swash::FontRef;

use super::{SharedMap, empty_map, file};

// asked in this order for letters the text's own font lacks, like japanese
const FALLBACK_FAMILIES: [&str; 2] = ["Noto Sans", "Noto Sans CJK JP"];

// a letter and the weight of the text it is in
type FallbackKey = (char, u16);

// the font found for each letter at each weight, none when no installed font has it
static FALLBACK_FOR: SharedMap<FallbackKey, Option<FontRef<'static>>> = LazyLock::new(empty_map);

/*
 * fonts count weight from 100 to 900, fontconfig has its own scale
 * where regular is 80 and bold is 200
 */
fn fontconfig_weight(weight: u16) -> i32 {
    let hundreds = (weight + 50) / 100;

    match hundreds {
        0 | 1 => 0,
        2 => 40,
        3 => 50,
        4 => 80,
        5 => 100,
        6 => 180,
        7 => 200,
        8 => 205,
        _ => 210,
    }
}

/*
 * fontconfig ranks having the letter above the family name, so any font
 * with it ties with noto; the first noto font that has it wins, any other
 * font with it is kept in case no noto font has the letter
 */
fn find_covering(letter: char, weight: u16) -> Option<FontRef<'static>> {
    let fontconfig = Fontconfig::new()?;

    let mut pattern = Pattern::new(&fontconfig).ok()?;

    for family in FALLBACK_FAMILIES {
        let family = std::ffi::CString::new(family).ok()?;

        pattern.add_string(c"family", &family).ok()?;
    }

    pattern
        .add_integer(c"weight", fontconfig_weight(weight))
        .ok()?;

    let mut charset = CharSet::new(&fontconfig).ok()?;

    charset.add_char(letter).ok()?;

    pattern.add_charset(charset).ok()?;

    let candidates = pattern.sort_fonts(UnicodeCoverage::Trim).ok()?;

    let mut other = None;

    for candidate in candidates.iter() {
        let Ok(family) = candidate.get_string(c"family") else {
            continue;
        };

        let Ok(font_path) = candidate.filename() else {
            continue;
        };

        let face_and_style = candidate.face_index().unwrap_or(0) as u32;

        let noto = family.starts_with("Noto");

        // opening a font file is slow, so a second non-noto font isn't tried
        if !noto && other.is_some() {
            continue;
        }

        let font = file::load(font_path, face_and_style);

        // the fonts with the letter come first, so the first without it ends the search
        if font.charmap().map(letter) == 0 {
            break;
        }

        if noto {
            return Some(font);
        }

        other = Some(font);
    }

    other
}

fn fallback(letter: char, weight: u16) -> Option<FontRef<'static>> {
    let key = (letter, weight);

    let mut fallback_for = FALLBACK_FOR.lock().unwrap_or_else(PoisonError::into_inner);

    if let Some(found) = fallback_for.get(&key) {
        return *found;
    }

    let found = find_covering(letter, weight);

    fallback_for.insert(key, found);

    found
}

// the font that has this letter: the text's own font first, then a fallback
pub fn with_letter(font: FontRef<'static>, letter: char) -> (FontRef<'static>, u16) {
    // glyph 0 is the font's placeholder box, which means it lacks the letter
    let id = font.charmap().map(letter);

    if id != 0 {
        return (font, id);
    }

    // so a letter in bold text gets a bold fallback
    let weight = font.attributes().weight().0;

    // a letter no font has draws as the text's own placeholder box
    let Some(fallback) = fallback(letter, weight) else {
        return (font, 0);
    };

    let id = fallback.charmap().map(letter);

    (fallback, id)
}
