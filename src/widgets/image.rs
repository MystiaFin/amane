use std::path::{Path, PathBuf};

use crate::graphics::image;

use super::fit::Fit;

pub struct Image {
    pub(crate) path: PathBuf,
    pub(crate) fit: Fit,

    // the size it is shrunk to cover when decoded, none keeps every pixel
    pub(crate) thumbnail: Option<(u32, u32)>,
}

impl Image {
    pub fn cover(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Cover,
            thumbnail: None,
        }
    }

    pub fn contain(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Contain,
            thumbnail: None,
        }
    }

    pub fn stretch(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Stretch,
            thumbnail: None,
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
     * starts decoding the file if nothing asked for it yet, and says whether
     * it is ready to draw; a file that can't be read never becomes ready
     */
    pub fn loaded(path: impl AsRef<Path>) -> bool {
        image::load(path.as_ref(), None).is_some()
    }
}
