use std::sync::{LazyLock, PoisonError};

use swash::FontRef;

use super::{SharedMap, empty_map, sliders};

// fonts already opened, labeled by their file and which font and style inside it
static FONTS_BY_FILE: SharedMap<String, FontRef<'static>> = LazyLock::new(empty_map);

static BYTES_BY_FILE: SharedMap<String, &'static [u8]> = LazyLock::new(empty_map);

pub fn open(font_path: &str, face_and_style: u32) -> FontRef<'static> {
    let mut bytes_by_file = BYTES_BY_FILE.lock().unwrap_or_else(PoisonError::into_inner);

    let bytes = match bytes_by_file.get(font_path) {
        // read before, maybe for another weight
        Some(bytes) => *bytes,

        None => {
            let read = std::fs::read(font_path).expect("failed to read font file");

            /*
             * fonts stay loaded until the program exits,
             * so leaking gives bytes that are valid forever
             */
            let leaked: &'static [u8] = Vec::leak(read);

            bytes_by_file.insert(String::from(font_path), leaked);

            leaked
        }
    };

    // fontconfig keeps a variable font's named style in the upper half, the file only knows the lower
    let face_number = face_and_style & 0xffff;

    let font = FontRef::from_index(bytes, face_number as usize).expect("failed to parse font");

    let style_number = face_and_style >> 16;

    sliders::remember(font, style_number);

    font
}

pub fn load(font_path: &str, face_and_style: u32) -> FontRef<'static> {
    let key = format!("{font_path} {face_and_style}");

    let mut fonts_by_file = FONTS_BY_FILE.lock().unwrap_or_else(PoisonError::into_inner);

    if let Some(font) = fonts_by_file.get(&key) {
        return *font;
    }

    let font = open(font_path, face_and_style);

    fonts_by_file.insert(key, font);

    font
}
