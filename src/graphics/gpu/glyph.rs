use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality};

use ttf_parser::Face;

use crate::graphics::Color;
use crate::graphics::letter;

use super::Gpu;

/*
 * a hover fade makes a new picture every frame for the letters it colors,
 * so the cache is emptied now and then instead of growing without end
 */
const MAX_GLYPHS: usize = 4096;

// one letter drawn once: its pixels and where they sit from the pen on the baseline
pub(super) struct Glyph {
    image: ImageData,
    left: f32,
    top: f32,
}

impl Glyph {
    pub(super) fn image(&self) -> &ImageData {
        &self.image
    }
}

// the face, the letter, the size and the color, which all change the pixels
pub(super) type GlyphKey = (usize, u16, u32, u32);

impl Gpu {
    pub(super) fn draw_glyph(
        &mut self,
        scene: &mut Scene,
        face: &'static Face<'static>,
        id: u16,
        size: f32,
        color: Color,
        x: f32,
        y: f32,
    ) {
        let key = (
            std::ptr::from_ref(face) as usize,
            id,
            size.to_bits(),
            u32::from_be_bytes([color.r, color.g, color.b, color.a]),
        );

        if self.glyphs.len() >= MAX_GLYPHS && !self.glyphs.contains_key(&key) {
            self.glyphs.clear();
        }

        let glyph = self
            .glyphs
            .entry(key)
            .or_insert_with(|| rasterize(face, id, size, color));

        // a space has no pixels
        let Some(glyph) = glyph else {
            return;
        };

        // letters sit on whole pixels, so the nearest pixel is exact and stays sharp
        let brush = ImageBrush::new(glyph.image.clone()).with_quality(ImageQuality::Low);

        let placement = Affine::translate((f64::from(x + glyph.left), f64::from(y + glyph.top)));

        scene.draw_image(&brush, placement);
    }
}

fn rasterize(face: &'static Face<'static>, id: u16, size: f32, color: Color) -> Option<Glyph> {
    let letter = letter::rasterize(face, id, size)?;

    let mut pixels = Vec::with_capacity(letter.coverage.len() * 4);

    for amount in letter.coverage {
        let color_alpha = u16::from(color.a);
        let coverage = u16::from(amount);

        let alpha = (color_alpha * coverage / 255) as u8;

        pixels.extend([color.r, color.g, color.b, alpha]);
    }

    let image = ImageData {
        data: Blob::from(pixels),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: letter.width,
        height: letter.height,
    };

    Some(Glyph {
        image,
        left: letter.left,
        top: letter.top,
    })
}
