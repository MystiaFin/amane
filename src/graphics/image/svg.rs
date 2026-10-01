use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

use super::Bitmap;

/*
 * drawn once at a fixed size and then cached like any image; drawing the
 * paths again every frame cost a full launcher of icons most of its frame
 */
// ponytail: one size for every svg, key the cache by shown size if a big svg looks soft
const LONGEST_SIDE: f32 = 256.0;

pub fn rasterize(bytes: &[u8]) -> Bitmap {
    let tree = Tree::from_data(bytes, &Options::default()).expect("failed to parse svg");

    let size = tree.size();

    let scale = LONGEST_SIDE / size.width().max(size.height());

    let width = (size.width() * scale).ceil() as u32;
    let height = (size.height() * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width, height).expect("failed to create svg pixmap");

    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    // tiny-skia keeps premultiplied alpha, a bitmap holds plain alpha
    let mut pixels = Vec::with_capacity(pixmap.data().len());

    for pixel in pixmap.pixels() {
        let plain = pixel.demultiply();

        pixels.extend([plain.red(), plain.green(), plain.blue(), plain.alpha()]);
    }

    Bitmap {
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_the_longest_side() {
        let bytes = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="red" fill-opacity="0.5"/></svg>"#;

        let image = rasterize(bytes);

        assert_eq!((image.width, image.height), (256, 128));

        // plain alpha, not premultiplied
        assert_eq!(&image.pixels[0..4], &[255, 0, 0, 128]);
    }
}
