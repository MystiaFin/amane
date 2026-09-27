use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use tiny_skia::{IntSize, Pixmap};
use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

const JPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];

static LOADED: LazyLock<Mutex<HashMap<PathBuf, &'static Pixmap>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn load(path: &Path) -> &'static Pixmap {
    let mut loaded = LOADED.lock().expect("failed to lock loaded images");

    if let Some(image) = loaded.get(path) {
        return image;
    }

    /*
     * images stay loaded until the program exits,
     * so leaking gives a reference that is valid forever
     */
    let image = Box::leak(Box::new(read(path)));

    loaded.insert(path.to_path_buf(), image);

    image
}

fn read(path: &Path) -> Pixmap {
    let bytes = std::fs::read(path).expect("failed to read image file");

    // the file's first bytes say its format, whatever its name ends in
    if bytes.starts_with(&PNG_SIGNATURE) {
        return Pixmap::decode_png(&bytes).expect("failed to decode png");
    }

    if bytes.starts_with(&JPEG_SIGNATURE) {
        return decode_jpeg(&bytes);
    }

    panic!("failed to load image: only png and jpeg are supported");
}

fn decode_jpeg(bytes: &[u8]) -> Pixmap {
    // the pixmap wants four channels, jpeg only stores three
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);

    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);

    let pixels = decoder.decode().expect("failed to decode jpeg");

    let info = decoder.info().expect("failed to read jpeg size");

    let size = IntSize::from_wh(u32::from(info.width), u32::from(info.height))
        .expect("failed to read jpeg size");

    // jpeg has no transparency, so the pixels already count as premultiplied
    Pixmap::from_vec(pixels, size).expect("failed to create pixmap")
}
