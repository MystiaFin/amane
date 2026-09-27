use std::path::PathBuf;

use super::fit::Fit;

pub struct Image {
    pub(crate) path: PathBuf,
    pub(crate) fit: Fit,
}

impl Image {
    pub fn cover(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Cover,
        }
    }

    pub fn contain(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Contain,
        }
    }

    pub fn stretch(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fit: Fit::Stretch,
        }
    }
}
