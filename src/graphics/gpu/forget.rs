use std::sync::Arc;

use crate::graphics::image::{self, Bitmap};

use super::Gpu;

/*
 * a window draws everything it shows every frame, so an image it
 * didn't draw this frame is gone from it, like an old wallpaper
 */
impl Gpu {
    // keeps the image alive while this window's copies of it are kept
    pub(super) fn note_shown(&mut self, image: &Arc<Bitmap>) {
        let key = Arc::as_ptr(image) as usize;

        self.shown.entry(key).or_insert_with(|| Arc::clone(image));

        self.drawn.insert(key);
    }

    pub(super) fn forget_unshown(&mut self) {
        let unshown: Vec<usize> = self
            .shown
            .keys()
            .filter(|key| !self.drawn.contains(key))
            .copied()
            .collect();

        for key in unshown {
            let Some(image) = self.shown.remove(&key) else {
                continue;
            };

            self.images.remove(&key);

            self.quads.forget(key);

            // held only here and by the loader, so no other window shows it either
            if Arc::strong_count(&image) == 2 {
                image::forget(&image);
            }
        }

        self.drawn.clear();
    }
}
