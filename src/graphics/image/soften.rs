use super::Bitmap;

/*
 * blurs the image by averaging each pixel with its neighbours up to
 * radius away, first along rows and then along columns; three rounds
 * of that look close to a gaussian blur
 */
pub fn soften(mut image: Bitmap, radius: u32) -> Bitmap {
    if radius == 0 {
        return image;
    }

    for _ in 0..3 {
        image.pixels = pass(&image, radius, 1, 0);
        image.pixels = pass(&image, radius, 0, 1);
    }

    image
}

// one box blur along x when step_x is 1, or along y when step_y is 1
fn pass(image: &Bitmap, radius: u32, step_x: u32, step_y: u32) -> Vec<u8> {
    let width = image.width as i64;
    let height = image.height as i64;
    let radius = radius as i64;

    let mut blurred = vec![0; image.pixels.len()];

    for y in 0..height {
        for x in 0..width {
            let mut total = [0u32; 4];
            let mut count = 0;

            for offset in -radius..=radius {
                // past the edge, the edge pixel repeats
                let sample_x = (x + offset * step_x as i64).clamp(0, width - 1);
                let sample_y = (y + offset * step_y as i64).clamp(0, height - 1);

                let index = ((sample_y * width + sample_x) * 4) as usize;

                for channel in 0..4 {
                    total[channel] += u32::from(image.pixels[index + channel]);
                }

                count += 1;
            }

            let index = ((y * width + x) * 4) as usize;

            for channel in 0..4 {
                blurred[index + channel] = (total[channel] / count) as u8;
            }
        }
    }

    blurred
}

#[cfg(test)]
mod tests {
    use super::*;

    // a hard black to white edge turns into a ramp, and flat areas stay flat
    #[test]
    fn edge_becomes_ramp() {
        let width = 16;

        let mut pixels = Vec::new();

        for x in 0..width {
            let value = if x < width / 2 { 0 } else { 255 };

            pixels.extend([value, value, value, 255]);
        }

        let image = soften(Bitmap { width, height: 1, pixels }, 2);

        let red: Vec<u8> = image.pixels.chunks(4).map(|pixel| pixel[0]).collect();

        assert_eq!(red[0], 0);
        assert_eq!(red[15], 255);

        assert!(red[7] > 0 && red[8] < 255, "the edge should soften: {red:?}");

        assert!(red.windows(2).all(|pair| pair[0] <= pair[1]), "should only rise: {red:?}");
    }
}
