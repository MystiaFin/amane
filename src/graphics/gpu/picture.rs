use vello::wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::graphics::image::Bitmap;

/*
 * an image as a gpu texture with every smaller size prepared too, so a big
 * icon shrunk into a small slot still looks smooth; colors are premultiplied,
 * which keeps soft edges from picking up dark fringes when they blend
 */
pub fn upload(device: &Device, queue: &Queue, image: &Bitmap) -> Texture {
    let mut levels = vec![premultiplied(image)];

    let mut width = image.width();
    let mut height = image.height();

    let mut sizes = vec![(width, height)];

    while width > 1 || height > 1 {
        let last = levels.last().expect("failed to find the last image size");

        let smaller = halve(last, width, height);

        width = (width / 2).max(1);
        height = (height / 2).max(1);

        levels.push(smaller);
        sizes.push((width, height));
    }

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: levels.len() as u32,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for (level, (pixels, (width, height))) in levels.iter().zip(sizes).enumerate() {
        let target = TexelCopyTextureInfo {
            texture: &texture,
            mip_level: level as u32,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        };

        let layout = TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        };

        let extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        queue.write_texture(target, pixels, layout, extent);
    }

    texture
}

fn premultiplied(image: &Bitmap) -> Vec<u8> {
    let mut pixels = image.pixels.clone();

    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);

        for channel in &mut pixel[..3] {
            *channel = (u16::from(*channel) * alpha / 255) as u8;
        }
    }

    pixels
}

// each pixel of the half size image is the average of the two by two it covers
fn halve(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let half_width = (width / 2).max(1);
    let half_height = (height / 2).max(1);

    let mut half = Vec::with_capacity((half_width * half_height * 4) as usize);

    for y in 0..half_height {
        for x in 0..half_width {
            for channel in 0..4 {
                let mut total = 0u32;

                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    // an odd edge repeats its last pixel
                    let source_x = (x * 2 + dx).min(width - 1);
                    let source_y = (y * 2 + dy).min(height - 1);

                    let index = ((source_y * width + source_x) * 4 + channel) as usize;

                    total += u32::from(pixels[index]);
                }

                half.push((total / 4) as u8);
            }
        }
    }

    half
}
