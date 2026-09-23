use std::sync::LazyLock;

use fontdue::{Font, FontSettings};

pub static FONT: LazyLock<Font> = LazyLock::new(|| {
    Font::from_bytes(
        include_bytes!("../../assets/font.ttf") as &[u8],
        FontSettings::default(),
    )
    .expect("failed to parse font")
});
