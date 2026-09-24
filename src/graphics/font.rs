use std::sync::LazyLock;

use fontconfig::Fontconfig;
use fontdue::{Font, FontSettings};

pub static FONT: LazyLock<Font> = LazyLock::new(|| {
    let fontconfig = Fontconfig::new().expect("failed to start fontconfig");

    let found = fontconfig
        .find("sans-serif", None)
        .expect("failed to find a sans-serif font");

    let bytes = std::fs::read(&found.path).expect("failed to read font file");

    // a .ttc file holds several fonts, the index says which one
    let settings = FontSettings {
        collection_index: found.index.unwrap_or(0) as u32,
        ..FontSettings::default()
    };

    Font::from_bytes(bytes, settings).expect("failed to parse font")
});
