use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use png::{ColorType, Decoder, Transformations};
use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

const JPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];

// rgba pixels with plain, not premultiplied, alpha
pub struct Bitmap {
    width: u32,
    height: u32,

    pub(crate) pixels: Vec<u8>,
}

impl Bitmap {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

static LOADED: LazyLock<Mutex<HashMap<PathBuf, &'static Bitmap>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn load(path: &Path) -> &'static Bitmap {
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

fn read(path: &Path) -> Bitmap {
    let bytes = std::fs::read(path).expect("failed to read image file");

    // the file's first bytes say its format, whatever its name ends in
    if bytes.starts_with(&PNG_SIGNATURE) {
        return decode_png(&bytes);
    }

    if bytes.starts_with(&JPEG_SIGNATURE) {
        return decode_jpeg(&bytes);
    }

    panic!("failed to load image: only png and jpeg are supported");
}

fn decode_png(bytes: &[u8]) -> Bitmap {
    let mut decoder = Decoder::new(Cursor::new(bytes));

    // palettes, tiny bit depths and 16 bit samples all turn into plain 8 bit channels
    decoder.set_transformations(Transformations::normalize_to_color8());

    let mut reader = decoder.read_info().expect("failed to decode png");

    let size = reader.output_buffer_size().expect("failed to measure png");

    let mut pixels = vec![0; size];

    let frame = reader
        .next_frame(&mut pixels)
        .expect("failed to decode png");

    pixels.truncate(frame.buffer_size());

    Bitmap {
        width: frame.width,
        height: frame.height,
        pixels: expand(&pixels, frame.color_type),
    }
}

// fills in the channels a png left out, so every pixel is red, green, blue, alpha
fn expand(pixels: &[u8], color_type: ColorType) -> Vec<u8> {
    match color_type {
        ColorType::Rgba => pixels.to_vec(),

        ColorType::Rgb => pixels
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),

        ColorType::GrayscaleAlpha => pixels
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),

        ColorType::Grayscale => pixels
            .iter()
            .flat_map(|&gray| [gray, gray, gray, 255])
            .collect(),

        ColorType::Indexed => panic!("failed to decode png: palette was not expanded"),
    }
}

fn decode_jpeg(bytes: &[u8]) -> Bitmap {
    // a bitmap has four channels, jpeg only stores three
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);

    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);

    let pixels = decoder.decode().expect("failed to decode jpeg");

    let info = decoder.info().expect("failed to read jpeg size");

    Bitmap {
        width: u32::from(info.width),
        height: u32::from(info.height),
        pixels,
    }
}
