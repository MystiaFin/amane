use std::sync::{LazyLock, Mutex, PoisonError};

use fontconfig::Fontconfig;
use swash::FontRef;

use crate::Weight;

use super::{SharedMap, empty_map, file};

static DEFAULT_FAMILY: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("sans-serif")));

// fonts already found by name, labeled by their family and weight
static FONTS_BY_FAMILY: SharedMap<String, FontRef<'static>> = LazyLock::new(empty_map);

pub fn set_default(family: &str) {
    let mut default_family = DEFAULT_FAMILY
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    *default_family = String::from(family);
}

fn find(family: &str, weight: Weight) -> FontRef<'static> {
    let fontconfig = Fontconfig::new().expect("failed to start fontconfig");

    // regular asks for no style, so plain text finds the same file it always did
    let style = match weight {
        Weight::Regular => None,
        other => Some(other.style()),
    };

    let found = fontconfig.find(family, style).expect("failed to find font");

    // a .ttc file holds several fonts, this says which one and at which named style
    let face_and_style = found.index.unwrap_or(0) as u32;

    file::open(&found.path.to_string_lossy(), face_and_style)
}

pub fn load_weighted(family: Option<&str>, weight: Weight) -> FontRef<'static> {
    let family = match family {
        Some(family) => String::from(family),
        None => DEFAULT_FAMILY
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone(),
    };

    // bold and regular are different files, so each weight is kept apart
    let key = format!("{family} {}", weight.style());

    let mut fonts_by_family = FONTS_BY_FAMILY.lock().unwrap_or_else(PoisonError::into_inner);

    if let Some(font) = fonts_by_family.get(&key) {
        return *font;
    }

    let font = find(&family, weight);

    fonts_by_family.insert(key, font);

    font
}
