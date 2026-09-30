use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use fontconfig::{CharSet, Fontconfig, Pattern, UnicodeCoverage};
use ttf_parser::{Face, GlyphId};

use crate::Weight;

// asked in this order for letters the text's own font lacks, like japanese
const FALLBACK_FAMILIES: [&str; 2] = ["Noto Sans", "Noto Sans CJK JP"];

static DEFAULT_FAMILY: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("sans-serif")));

static LOADED: LazyLock<Mutex<HashMap<String, &'static Face<'static>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// the font found for each letter, none when no installed font has it
static FALLBACKS: LazyLock<Mutex<HashMap<char, Option<&'static Face<'static>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_default(family: &str) {
    let mut default_family = DEFAULT_FAMILY.lock().expect("failed to lock default font");

    *default_family = String::from(family);
}

pub fn load(family: Option<&str>) -> &'static Face<'static> {
    load_weighted(family, Weight::Regular)
}

pub fn load_weighted(family: Option<&str>, weight: Weight) -> &'static Face<'static> {
    let family = match family {
        Some(family) => String::from(family),
        None => DEFAULT_FAMILY
            .lock()
            .expect("failed to lock default font")
            .clone(),
    };

    // bold and regular are different files, so each weight is kept apart
    let key = format!("{family} {}", weight.style());

    let mut loaded = LOADED.lock().expect("failed to lock loaded fonts");

    if let Some(font) = loaded.get(&key) {
        return font;
    }

    /*
     * fonts stay loaded until the program exits,
     * so leaking gives a reference that is valid forever
     */
    let font = Box::leak(Box::new(read(&family, weight)));

    loaded.insert(key, font);

    font
}

fn read(family: &str, weight: Weight) -> Face<'static> {
    let fontconfig = Fontconfig::new().expect("failed to start fontconfig");

    // regular asks for no style, so plain text finds the same file it always did
    let style = match weight {
        Weight::Regular => None,
        other => Some(other.style()),
    };

    let found = fontconfig.find(family, style).expect("failed to find font");

    // a .ttc file holds several fonts, the index says which one
    let index = found.index.unwrap_or(0) as u32;

    open(&found.path.to_string_lossy(), index)
}

fn open(path: &str, index: u32) -> Face<'static> {
    let bytes = std::fs::read(path).expect("failed to read font file");

    // the face borrows the bytes, so they have to live as long as it
    let bytes = Vec::leak(bytes);

    // fontconfig keeps a variable font's named style in the upper half, the file only knows the lower
    let face_index = index & 0xffff;

    Face::parse(bytes, face_index).expect("failed to parse font")
}

// the font that has this letter: the text's own font first, then a fallback
pub fn with_letter<'a>(font: &'a Face<'a>, letter: char) -> (&'a Face<'a>, GlyphId) {
    if let Some(id) = font.glyph_index(letter) {
        return (font, id);
    }

    // a letter no font has draws as the text's own placeholder box
    let Some(fallback) = fallback(letter) else {
        return (font, GlyphId(0));
    };

    let id = fallback.glyph_index(letter).unwrap_or_default();

    (fallback, id)
}

fn fallback(letter: char) -> Option<&'static Face<'static>> {
    let mut fallbacks = FALLBACKS.lock().expect("failed to lock fallback fonts");

    if let Some(found) = fallbacks.get(&letter) {
        return *found;
    }

    let found = find_covering(letter);

    fallbacks.insert(letter, found);

    found
}

/*
 * fontconfig ranks having the letter above the family name, so any font
 * with it ties with noto; the first noto font that has it wins, any other
 * font with it is kept in case no noto font has the letter
 */
fn find_covering(letter: char) -> Option<&'static Face<'static>> {
    let fontconfig = Fontconfig::new()?;

    let mut pattern = Pattern::new(&fontconfig).ok()?;

    for family in FALLBACK_FAMILIES {
        let family = std::ffi::CString::new(family).ok()?;

        pattern.add_string(c"family", &family).ok()?;
    }

    let mut charset = CharSet::new(&fontconfig).ok()?;

    charset.add_char(letter).ok()?;

    pattern.add_charset(charset).ok()?;

    let candidates = pattern.sort_fonts(UnicodeCoverage::Trim).ok()?;

    let mut other = None;

    for candidate in candidates.iter() {
        let Ok(family) = candidate.get_string(c"family") else {
            continue;
        };

        let Ok(path) = candidate.filename() else {
            continue;
        };

        let index = candidate.face_index().unwrap_or(0) as u32;

        let noto = family.starts_with("Noto");

        // opening a font file is slow, so a second non-noto font isn't tried
        if !noto && other.is_some() {
            continue;
        }

        let face = load_file(path, index);

        // the fonts with the letter come first, so the first without it ends the search
        if face.glyph_index(letter).is_none() {
            break;
        }

        if noto {
            return Some(face);
        }

        other = Some(face);
    }

    other
}

fn load_file(path: &str, index: u32) -> &'static Face<'static> {
    let key = format!("{path} {index}");

    let mut loaded = LOADED.lock().expect("failed to lock loaded fonts");

    if let Some(face) = loaded.get(&key) {
        return face;
    }

    let face = Box::leak(Box::new(open(path, index)));

    loaded.insert(key, face);

    face
}
