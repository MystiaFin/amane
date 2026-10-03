use std::path::{Path, PathBuf};

use crate::graphics::{Rect, image};

pub struct Image {
    pub(crate) path: PathBuf,
    pub(crate) fit: Fit,

    // the size it is shrunk to cover when decoded, none keeps every pixel
    pub(crate) thumbnail: Option<(u32, u32)>,

    // how far the decoded copy is blurred, in its own pixels
    pub(crate) blur: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    // fills the rectangle exactly, bending the image's shape to match
    Stretch,

    // fills the whole rectangle, cutting off what spills past the edges
    Cover,

    // shows the whole image, leaving the rest of the rectangle uncovered
    Contain,
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

        image::load(path, None, 0).is_some()
    }
}

impl Fit {
    pub fn place(self, area: Rect, image_width: f32, image_height: f32) -> Rect {
        let horizontal_scale = area.width / image_width;
        let vertical_scale = area.height / image_height;

        let scale = match self {
            Fit::Stretch => return area,
            Fit::Cover => f32::max(horizontal_scale, vertical_scale),
            Fit::Contain => f32::min(horizontal_scale, vertical_scale),
        };

        let width = image_width * scale;
        let height = image_height * scale;

        let x = area.x + (area.width - width) / 2.0;
        let y = area.y + (area.height - height) / 2.0;

        Rect::new(x, y, width, height)
    }
}
