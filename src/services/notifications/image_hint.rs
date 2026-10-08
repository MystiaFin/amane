use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

use png::{BitDepth, ColorType, Encoder};

use crate::Value;

// the spec renamed this hint twice, senders still use all three names
const DATA_HINTS: [&str; 3] = ["image-data", "image_data", "icon_data"];

const PATH_HINTS: [&str; 2] = ["image-path", "image_path"];

const FOLDER: &str = "amane-notifications";

// each saved image gets a new name, so a replaced notification never shows a cached old one
static NEXT_NAME: AtomicU32 = AtomicU32::new(0);

/*
 * raw pixels are written to a png, so they are drawn like any image
 * file; a sender's own image path comes back as it was sent
 */
pub fn read(hints: &Value) -> String {
    for name in DATA_HINTS {
        if let Some(path) = save(hints.get(name)) {
            return path.to_string_lossy().into_owned();
        }
    }

    for name in PATH_HINTS {
        let path = hints.get(name).text();

        if !path.is_empty() {
            return String::from(path);
        }
    }

    String::new()
}

// only images saved here are removed, never a file the sender owns
pub fn remove(image: &str) {
    let path = Path::new(image);

    if path.parent() == Some(&folder()) {
        let _ = fs::remove_file(path);
    }
}

fn folder() -> PathBuf {
    let runtime_directory =
        env::var_os("XDG_RUNTIME_DIR").unwrap_or_else(|| env::temp_dir().into());

    PathBuf::from(runtime_directory).join(FOLDER)
}

// the hint is (width, height, rowstride, has_alpha, bits_per_sample, channels, data)
fn save(hint: &Value) -> Option<PathBuf> {
    let [width, height, rowstride, _has_alpha, bits, channels, data] = hint.list() else {
        return None;
    };

    let width = width.number() as usize;
    let height = height.number() as usize;
    let rowstride = rowstride.number() as usize;
    let channels = channels.number() as usize;

    // the spec only allows 8 bit rgb or rgba
    if bits.number() != 8.0 || !(channels == 3 || channels == 4) || width == 0 || height == 0 {
        return None;
    }

    let data = data.list();

    let mut pixels = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let start = y * rowstride + x * channels;

            let pixel = data.get(start..start + channels)?;

            for channel in pixel {
                pixels.push(channel.number() as u8);
            }

            if channels == 3 {
                pixels.push(255);
            }
        }
    }

    let folder = folder();

    fs::create_dir_all(&folder).ok()?;

    let number = NEXT_NAME.fetch_add(1, Ordering::Relaxed);

    let path = folder.join(format!("{}-{number}.png", process::id()));

    let file = BufWriter::new(File::create(&path).ok()?);

    let mut encoder = Encoder::new(file, width as u32, height as u32);

    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    encoder
        .write_header()
        .ok()?
        .write_image_data(&pixels)
        .ok()?;

    Some(path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::graphics::image;

    #[test]
    fn saves_image_data_as_png() {
        let number = |value: f64| Value::Number(value);

        // two rgb pixels in a row padded to 8 bytes
        let bytes = [255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 9.0, 9.0];

        let hint = Value::List(vec![
            number(2.0),
            number(1.0),
            number(8.0),
            Value::Bool(false),
            number(8.0),
            number(3.0),
            Value::List(bytes.map(number).to_vec()),
        ]);

        let hints = Value::Map(BTreeMap::from([(String::from("image-data"), hint)]));

        let path = read(&hints);

        let bitmap = image::read(Path::new(&path)).expect("failed to read saved image");

        remove(&path);

        assert_eq!(*bitmap.pixels, [255, 0, 0, 255, 0, 255, 0, 255]);
        assert!(!Path::new(&path).exists());
    }
}
