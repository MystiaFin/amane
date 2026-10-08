mod shrink;
mod soften;
mod svg;

use std::any::TypeId;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::thread;

use png::{ColorType, Decoder, Transformations};
use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;

use crate::changes;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

const JPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];

// rgba pixels with plain, not premultiplied, alpha
pub struct Bitmap {
    width: u32,
    height: u32,

    // shared with the gpu, which draws from the same pixels instead of a copy
    pub pixels: Arc<Vec<u8>>,
}

impl Bitmap {
    // one clear pixel, for a texture slot that has to hold something
    pub fn empty() -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: Arc::new(vec![0; 4]),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

/*
 * what a window waiting on an image counts as having read, so a finished
 * decode only draws the windows still missing a picture, not every window
 */
struct Decoded;

// a file, the size it is shrunk to cover (none for its own size), and how far it is blurred
type Key = (PathBuf, Option<(u32, u32)>, u32);

// every image asked for: none while it is still decoding, or when it could not be read
static LOADED: LazyLock<Mutex<HashMap<Key, Option<Arc<Bitmap>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/*
 * decoding a large image takes long enough to drop frames, so it happens
 * on its own thread; until it is done this gives none and the image is
 * left out, then the window is woken to draw it
 */
pub fn load(path: &Path, cover: Option<(u32, u32)>, blur: u32) -> Option<Arc<Bitmap>> {
    let mut loaded = LOADED.lock().unwrap_or_else(PoisonError::into_inner);

    let key = (path.to_path_buf(), cover, blur);

    if let Some(image) = loaded.get(&key) {
        if image.is_none() {
            changes::note_read(TypeId::of::<Decoded>());
        }

        return image.clone();
    }

    loaded.insert(key.clone(), None);

    thread::spawn(move || decode(key));

    changes::note_read(TypeId::of::<Decoded>());

    None
}

fn decode(key: Key) {
    let (path, cover, blur) = &key;

    // a broken file stays empty instead of taking the shell down
    let Some(image) = read(path) else {
        return;
    };

    // only the small copy is kept, the full size one is dropped here
    let image = match cover {
        Some((width, height)) => shrink::to_cover(image, *width, *height),
        None => image,
    };

    // blurred after shrinking, where there are far fewer pixels to average
    let image = soften::soften(image, *blur);

    LOADED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(key, Some(Arc::new(image)));

    changes::mark(TypeId::of::<Decoded>());
}

/*
 * frees a full size image no window shows anymore; small copies stay,
 * they cost little and decoding them again is slow
 */
pub fn forget(image: &Arc<Bitmap>) {
    let mut loaded = LOADED.lock().unwrap_or_else(PoisonError::into_inner);

    loaded.retain(|(_, cover, _), kept| {
        let same = kept.as_ref().is_some_and(|kept| Arc::ptr_eq(kept, image));

        cover.is_some() || !same
    });
}

// none for a file that can't be read or decoded, like one still being written
pub fn read(path: &Path) -> Option<Bitmap> {
    let bytes = std::fs::read(path).ok()?;

    // the file's first bytes say its format, whatever its name ends in
    if bytes.starts_with(&PNG_SIGNATURE) {
        return decode_png(&bytes);
    }

    if bytes.starts_with(&JPEG_SIGNATURE) {
        return decode_jpeg(&bytes);
    }

    if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("svg")) {
        return svg::rasterize(&bytes);
    }

    // only png, jpeg and svg are supported
    None
}

fn decode_png(bytes: &[u8]) -> Option<Bitmap> {
    let mut decoder = Decoder::new(Cursor::new(bytes));

    // palettes, tiny bit depths and 16 bit samples all turn into plain 8 bit channels
    decoder.set_transformations(Transformations::normalize_to_color8());

    let mut reader = decoder.read_info().ok()?;

    let size = reader.output_buffer_size()?;

    let mut pixels = vec![0; size];

    let frame = reader.next_frame(&mut pixels).ok()?;

    pixels.truncate(frame.buffer_size());

    let bitmap = Bitmap {
        width: frame.width,
        height: frame.height,
        pixels: Arc::new(expand(&pixels, frame.color_type)),
    };

    Some(bitmap)
}

// fills in the channels a png left out, so every pixel is red, green, blue, alpha
fn expand(pixels: &[u8], color_type: ColorType) -> Vec<u8> {
    match color_type {
        ColorType::Rgba => pixels.to_vec(),

        ColorType::Rgb => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),

        ColorType::GrayscaleAlpha => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),

        ColorType::Grayscale => pixels
            .iter()
            .flat_map(|&gray| [gray, gray, gray, 255])
            .collect(),

        ColorType::Indexed => panic!("failed to decode png: palette was not expanded"),
    }
}

fn decode_jpeg(bytes: &[u8]) -> Option<Bitmap> {
    // a bitmap has four channels, jpeg only stores three
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);

    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);

    let pixels = decoder.decode().ok()?;

    let info = decoder.info()?;

    let bitmap = Bitmap {
        width: u32::from(info.width),
        height: u32::from(info.height),
        pixels: Arc::new(pixels),
    };

    Some(bitmap)
}
