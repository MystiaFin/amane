use crate::Color;

// one group of similar pixels, it becomes one palette color
struct Bucket {
    pixels: Vec<[u8; 3]>,
}

impl Bucket {
    // the channel (red, green or blue) whose values spread the most, and how far
    fn widest_channel(&self) -> (usize, u8) {
        let mut widest = 0;
        let mut widest_range = 0;

        for channel in 0..3 {
            let mut lowest = u8::MAX;
            let mut highest = u8::MIN;

            for pixel in &self.pixels {
                lowest = lowest.min(pixel[channel]);
                highest = highest.max(pixel[channel]);
            }

            let range = highest.saturating_sub(lowest);

            if range > widest_range {
                widest = channel;
                widest_range = range;
            }
        }

        (widest, widest_range)
    }

    fn average(&self) -> Color {
        let mut total = [0u64; 3];

        for pixel in &self.pixels {
            for channel in 0..3 {
                total[channel] += u64::from(pixel[channel]);
            }
        }

        let count = self.pixels.len() as u64;

        let red = (total[0] / count) as u8;
        let green = (total[1] / count) as u8;
        let blue = (total[2] / count) as u8;

        Color::rgb(red, green, blue)
    }
}

/*
 * median cut: start with every pixel in one bucket, then keep
 * splitting the bucket with the widest color spread in half along
 * that channel, until there are `count` buckets. each bucket's
 * average is one color, and its size is how much of the image it covers
 */
pub fn quantize(pixels: Vec<[u8; 3]>, count: usize) -> Vec<Color> {
    if pixels.is_empty() || count == 0 {
        return Vec::new();
    }

    let mut buckets = vec![Bucket { pixels }];

    while buckets.len() < count {
        let Some(index) = widest_bucket(&buckets) else {
            break;
        };

        let bucket = buckets.swap_remove(index);

        let (lower, upper) = split(bucket);

        buckets.push(lower);
        buckets.push(upper);
    }

    // most pixels first, so the first color is the one the image shows most
    buckets.sort_by_key(|bucket| std::cmp::Reverse(bucket.pixels.len()));

    buckets.iter().map(Bucket::average).collect()
}

// none when every bucket is a single flat color, so splitting would change nothing
fn widest_bucket(buckets: &[Bucket]) -> Option<usize> {
    let mut widest = None;
    let mut widest_range = 0;

    for (index, bucket) in buckets.iter().enumerate() {
        if bucket.pixels.len() < 2 {
            continue;
        }

        let (_, range) = bucket.widest_channel();

        if range > widest_range {
            widest = Some(index);
            widest_range = range;
        }
    }

    widest
}

fn split(mut bucket: Bucket) -> (Bucket, Bucket) {
    let (channel, _) = bucket.widest_channel();

    bucket.pixels.sort_unstable_by_key(|pixel| pixel[channel]);

    let middle = bucket.pixels.len() / 2;

    let upper = bucket.pixels.split_off(middle);

    (bucket, Bucket { pixels: upper })
}
