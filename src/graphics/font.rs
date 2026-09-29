use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use fontconfig::Fontconfig;
use fontdue::{Font, FontSettings};
use ttf_parser::Face;

use crate::Weight;

static DEFAULT_FAMILY: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("sans-serif")));

static LOADED: LazyLock<Mutex<HashMap<String, &'static Face<'static>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// the same fonts again for turning letters into pixels, keyed by where each face lives
static RASTERIZERS: LazyLock<Mutex<HashMap<usize, &'static Font>>> =
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
    let (face, bytes, index) = read(&family, weight);

    let font = Box::leak(Box::new(face));

    let settings = FontSettings {
        collection_index: index,
        ..FontSettings::default()
    };

    let rasterizer = Font::from_bytes(bytes, settings).expect("failed to read font for drawing");

    RASTERIZERS
        .lock()
        .expect("failed to lock font rasterizers")
        .insert(
            std::ptr::from_ref(font) as usize,
            Box::leak(Box::new(rasterizer)),
        );

    loaded.insert(key, font);

    font
}

// the rasterizer for a face that load_weighted handed out
pub fn rasterizer(face: &Face) -> &'static Font {
    let rasterizers = RASTERIZERS.lock().expect("failed to lock font rasterizers");

    rasterizers
        .get(&(std::ptr::from_ref(face) as usize))
        .expect("failed to find the rasterizer for a font")
}

fn read(family: &str, weight: Weight) -> (Face<'static>, &'static [u8], u32) {
    let fontconfig = Fontconfig::new().expect("failed to start fontconfig");

    // regular asks for no style, so plain text finds the same file it always did
    let style = match weight {
        Weight::Regular => None,
        other => Some(other.style()),
    };

    let found = fontconfig.find(family, style).expect("failed to find font");

    let bytes = std::fs::read(&found.path).expect("failed to read font file");

    // the face borrows the bytes, so they have to live as long as it
    let bytes = Vec::leak(bytes);

    // a .ttc file holds several fonts, the index says which one
    let index = found.index.unwrap_or(0) as u32;

    let face = Face::parse(bytes, index).expect("failed to parse font");

    (face, bytes, index)
}
