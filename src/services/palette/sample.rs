use crate::graphics::image::Bitmap;

// about 64 by 64 pixels is plenty to find the main colors, and quick to sort
const SIDE: u32 = 64;

// below this the pixel is mostly see-through, so it says little about the image
const MIN_ALPHA: u8 = 128;

pub fn pixels(image: &Bitmap) -> Vec<[u8; 3]> {
    let longest = image.width().max(image.height());

    // every step-th pixel in both directions, so a 4k image is read 60 times faster
    let step = (longest / SIDE).max(1);

    let mut picked = Vec::new();

    for y in (0..image.height()).step_by(step as usize) {
        for x in (0..image.width()).step_by(step as usize) {
            let start = ((y * image.width() + x) * 4) as usize;

            let pixel = &image.pixels[start..start + 4];

            if pixel[3] < MIN_ALPHA {
                continue;
            }

            picked.push([pixel[0], pixel[1], pixel[2]]);
        }
    }

    picked
}
