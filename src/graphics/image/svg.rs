use std::sync::Arc;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

use super::Bitmap;

/*
 * drawn once at a fixed size and then cached like any image; drawing the
 * paths again every frame cost a full launcher of icons most of its frame
 */
// one size for every svg; keying the cache by the shown size would sharpen a big svg that looks soft
const LONGEST_SIDE: f32 = 256.0;

pub fn rasterize(bytes: &[u8]) -> Option<Bitmap> {
    let tree = Tree::from_data(bytes, &Options::default()).ok()?;

    let size = tree.size();

    let scale = LONGEST_SIDE / size.width().max(size.height());

    let width = (size.width() * scale).ceil() as u32;
    let height = (size.height() * scale).ceil() as u32;

    // none for an svg with no size to draw at
    let mut pixmap = Pixmap::new(width, height)?;

    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    // tiny-skia keeps premultiplied alpha, a bitmap holds plain alpha
    let mut pixels = Vec::with_capacity(pixmap.data().len());

    for pixel in pixmap.pixels() {
        let plain = pixel.demultiply();

        pixels.extend([plain.red(), plain.green(), plain.blue(), plain.alpha()]);
    }

    let bitmap = Bitmap {
        width,
        height,
        pixels: Arc::new(pixels),
    };

    Some(bitmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_the_longest_side() {
        let bytes = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="red" fill-opacity="0.5"/></svg>"#;

        let image = rasterize(bytes).expect("failed to rasterize the test svg");

        assert_eq!((image.width, image.height), (256, 128));

        // plain alpha, not premultiplied
        assert_eq!(&image.pixels[0..4], &[255, 0, 0, 128]);
    }
}
