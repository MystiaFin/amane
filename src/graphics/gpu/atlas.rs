use std::collections::HashMap;

use ttf_parser::Face;
use vello::wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::graphics::font;

const SIZE: u32 = 1024;

// a gap between letters, so none reads a neighbour's pixels
const PADDING: u32 = 1;

// where one letter's pixels are in the atlas, and where they sit from the pen on the baseline
#[derive(Clone, Copy)]
pub struct Letter {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub left: f32,
    pub top: f32,
}

// the face, the letter and its size in pixels
type Key = (usize, u16, u32);

/*
 * every letter drawn so far, in one texture of coverage values; letters fill
 * it in rows, and a full atlas starts over empty and fills again from what is shown
 */
pub struct Atlas {
    texture: Texture,

    letters: HashMap<Key, Option<Letter>>,

    // the row being filled: its top, how far along it is and its tallest letter
    row_top: u32,
    row_x: u32,
    row_height: u32,
}

impl Atlas {
    pub fn new(device: &Device) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            texture,
            letters: HashMap::new(),
            row_top: 0,
            row_x: 0,
            row_height: 0,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    // none for a letter without pixels, like a space
    pub fn letter(
        &mut self,
        queue: &Queue,
        face: &'static Face<'static>,
        id: u16,
        size: f32,
    ) -> Option<Letter> {
        let key = (std::ptr::from_ref(face) as usize, id, size.to_bits());

        if let Some(letter) = self.letters.get(&key) {
            return *letter;
        }

        let (metrics, coverage) = font::rasterizer(face).rasterize_indexed(id, size);

        let width = metrics.width as u32;
        let height = metrics.height as u32;

        if width == 0 || height == 0 {
            self.letters.insert(key, None);

            return None;
        }

        // too big for the atlas at all
        if width + PADDING > SIZE || height + PADDING > SIZE {
            return None;
        }

        let (x, y) = self.place(width, height);

        let target = TexelCopyTextureInfo {
            texture: &self.texture,
            mip_level: 0,
            origin: Origin3d { x, y, z: 0 },
            aspect: TextureAspect::All,
        };

        let layout = TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width),
            rows_per_image: Some(height),
        };

        let extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        queue.write_texture(target, &coverage, layout, extent);

        // ymin counts up from the baseline to the picture's bottom edge, the screen counts down
        let letter = Letter {
            x,
            y,
            width,
            height,
            left: metrics.xmin as f32,
            top: -(metrics.ymin as f32 + metrics.height as f32),
        };

        self.letters.insert(key, Some(letter));

        Some(letter)
    }

    // the next free spot in the rows, starting over once the atlas is full
    fn place(&mut self, width: u32, height: u32) -> (u32, u32) {
        if self.row_x + width + PADDING > SIZE {
            self.row_top += self.row_height + PADDING;
            self.row_x = 0;
            self.row_height = 0;
        }

        if self.row_top + height + PADDING > SIZE {
            self.letters.clear();

            self.row_top = 0;
            self.row_x = 0;
            self.row_height = 0;
        }

        let spot = (self.row_x, self.row_top);

        self.row_x += width + PADDING;
        self.row_height = self.row_height.max(height);

        spot
    }
}
