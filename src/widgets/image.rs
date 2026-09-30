use std::path::{Path, PathBuf};

use crate::graphics::{image, svg};

use super::fit::Fit;

pub struct Image {
    pub(crate) path: PathBuf,
    pub(crate) fit: Fit,

    // the size it is shrunk to cover when decoded, none keeps every pixel
    pub(crate) thumbnail: Option<(u32, u32)>,

    // how far the decoded copy is blurred, in its own pixels
    pub(crate) blur: u32,
}

impl Image {
    pub fn cover(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Cover,
            thumbnail: None,
            blur: 0,
        }
    }

    pub fn contain(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Contain,
            thumbnail: None,
            blur: 0,
        }
    }

    pub fn stretch(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Stretch,
            thumbnail: None,
            blur: 0,
        }
    }

    /*
     * keeps a small copy big enough to cover width by height, for pictures
     * shown far smaller than their file, like a list of wallpapers
     */
    pub fn thumbnail(mut self, width: u32, height: u32) -> Self {
        self.thumbnail = Some((width, height));

        self
    }

    /*
     * blurs the decoded copy once, radius counted in its own pixels; with a
     * small thumbnail stretched far bigger, like a blurred backdrop, it
     * looks smooth instead of blocky and costs nothing per frame
     */
    pub fn blurred(mut self, radius: u32) -> Self {
        self.blur = radius;

        self
    }

    /*
     * starts decoding the file if nothing asked for it yet, and says whether
     * it is ready to draw; a file that can't be read never becomes ready
     */
    pub fn loaded(path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        if svg::is_svg(path) {
            return svg::load(path).is_some();
        }

        image::load(path, None, 0).is_some()
    }
}
