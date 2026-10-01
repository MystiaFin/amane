use std::sync::Arc;

use super::Bitmap;

/*
 * scales the image down until it only just covers width by height, each
 * new pixel the average of the pixels it stands for; an image already that
 * small is left as it is
 */
pub fn to_cover(image: Bitmap, width: u32, height: u32) -> Bitmap {
    let scale_x = width as f32 / image.width as f32;
    let scale_y = height as f32 / image.height as f32;

    let scale = f32::max(scale_x, scale_y);

    if scale >= 1.0 {
        return image;
    }

    let new_width = ((image.width as f32 * scale).ceil() as u32).max(1);
    let new_height = ((image.height as f32 * scale).ceil() as u32).max(1);

    let mut pixels = Vec::with_capacity((new_width * new_height * 4) as usize);

    for y in 0..new_height {
        let top = y * image.height / new_height;
        let bottom = ((y + 1) * image.height / new_height).max(top + 1);

        for x in 0..new_width {
            let left = x * image.width / new_width;
            let right = ((x + 1) * image.width / new_width).max(left + 1);

            pixels.extend(average(&image, left, right, top, bottom));
        }
    }

    Bitmap {
        width: new_width,
        height: new_height,
        pixels: Arc::new(pixels),
    }
}

// the mean of every channel over the pixels from left to right and top to bottom
fn average(image: &Bitmap, left: u32, right: u32, top: u32, bottom: u32) -> [u8; 4] {
    let mut total = [0u32; 4];

    for y in top..bottom {
        for x in left..right {
            let start = ((y * image.width + x) * 4) as usize;

            for channel in 0..4 {
                total[channel] += u32::from(image.pixels[start + channel]);
            }
        }
    }

    let count = (right - left) * (bottom - top);

    total.map(|sum| (sum / count) as u8)
}
