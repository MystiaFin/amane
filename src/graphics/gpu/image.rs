use std::sync::Arc;

use vello::Scene;
use vello::peniko::{Blob, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality};

use crate::graphics::image::Bitmap;
use crate::graphics::{Path, Transform};

use super::Gpu;
use super::convert::{affine, bezier};

impl Gpu {
    pub(super) fn draw_image(
        &mut self,
        scene: &mut Scene,
        image: Arc<Bitmap>,
        transform: Transform,
        clip: &Path,
        clip_transform: Transform,
    ) {
        // bicubic keeps a large image smooth when it shrinks to fit
        self.note_shown(&image);

        let brush = ImageBrush::new(self.image(&image)).with_quality(ImageQuality::High);

        scene.push_clip_layer(Fill::NonZero, affine(clip_transform), &bezier(clip));

        scene.draw_image(&brush, affine(transform));

        scene.pop_layer();
    }

    /*
     * vello throws its image atlas away after a scene without images,
     * but still counts the images in it as uploaded, so they are sent again
     */
    pub(super) fn keep_atlas(&mut self, scene: &Scene) {
        if scene.encoding().resources.patches.is_empty() {
            self.atlas_dropped = true;

            return;
        }

        if !self.atlas_dropped {
            return;
        }

        for image in self.images.values() {
            self.vello.mark_override_image_dirty(image);
        }

        for glyph in self.glyphs.values().flatten() {
            self.vello.mark_override_image_dirty(glyph.image());
        }

        self.atlas_dropped = false;
    }

    // a shown image stays where it is, so where it lives says which image it is
    fn image(&mut self, image: &Bitmap) -> ImageData {
        let key = std::ptr::from_ref(image) as usize;

        let converted = self.images.entry(key).or_insert_with(|| ImageData {
            data: Blob::from(image.pixels.clone()),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: image.width(),
            height: image.height(),
        });

        converted.clone()
    }
}
