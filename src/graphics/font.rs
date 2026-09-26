use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use fontconfig::Fontconfig;
use ttf_parser::Face;

static DEFAULT_FAMILY: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("sans-serif")));

static LOADED: LazyLock<Mutex<HashMap<String, &'static Face<'static>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_default(family: &str) {
    let mut default_family = DEFAULT_FAMILY.lock().expect("failed to lock default font");

    *default_family = String::from(family);
}

pub fn load(family: Option<&str>) -> &'static Face<'static> {
    let family = match family {
        Some(family) => String::from(family),
        None => DEFAULT_FAMILY
            .lock()
            .expect("failed to lock default font")
            .clone(),
    };

    let mut loaded = LOADED.lock().expect("failed to lock loaded fonts");

    if let Some(font) = loaded.get(&family) {
        return font;
    }

    /*
     * fonts stay loaded until the program exits,
     * so leaking gives a reference that is valid forever
     */
    let font = Box::leak(Box::new(read(&family)));

    loaded.insert(family, font);

    font
}

fn read(family: &str) -> Face<'static> {
    let fontconfig = Fontconfig::new().expect("failed to start fontconfig");

    let found = fontconfig.find(family, None).expect("failed to find font");

    let bytes = std::fs::read(&found.path).expect("failed to read font file");

    // the face borrows the bytes, so they have to live as long as it
    let bytes = Vec::leak(bytes);

    // a .ttc file holds several fonts, the index says which one
    let index = found.index.unwrap_or(0) as u32;

    Face::parse(bytes, index).expect("failed to parse font")
}
