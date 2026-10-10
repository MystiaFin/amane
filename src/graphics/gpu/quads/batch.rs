use vello::wgpu::{Device, Queue};

use ttf_parser::Face;

use crate::graphics::gpu::picture;
use crate::graphics::image::Bitmap;
use crate::graphics::{Area, Color, Corners};

use super::clip::NO_CLIP;
use super::pipeline::picture_group;
use super::{Clips, LETTER, PICTURE, QUAD_SIZE, Quads, Run, SHAPE};

impl Quads {
    // area and radius in canvas pixels
    pub fn rectangle(&mut self, area: Area, radius: Corners, color: Color, clips: Clips) {
        self.push(area, (radius, 0.0), color, clips, SHAPE, [0.0; 4], None);
    }

    pub fn border(
        &mut self,
        area: Area,
        radius: Corners,
        thickness: f32,
        color: Color,
        clips: Clips,
    ) {
        self.push(
            area,
            (radius, thickness),
            color,
            clips,
            SHAPE,
            [0.0; 4],
            None,
        );
    }

    // x and y are where the letter's baseline starts, on a whole pixel
    #[allow(clippy::too_many_arguments)]
    pub fn letter(
        &mut self,
        queue: &Queue,
        face: &'static Face<'static>,
        id: u16,
        size: f32,
        color: Color,
        x: f32,
        y: f32,
        clips: Clips,
    ) {
        let Some(letter) = self.atlas.letter(queue, face, id, size) else {
            return;
        };

        let area = Area::new(
            x + letter.left,
            y + letter.top,
            letter.width as f32,
            letter.height as f32,
        );

        let texels = [
            letter.x as f32,
            letter.y as f32,
            (letter.x + letter.width) as f32,
            (letter.y + letter.height) as f32,
        ];

        self.push(
            area,
            (Corners::default(), 0.0),
            color,
            clips,
            LETTER,
            texels,
            None,
        );
    }

    pub fn forget(&mut self, picture: usize) {
        self.pictures.remove(&picture);
    }

    // the whole image stretched over area, which the clip usually trims
    pub fn picture(
        &mut self,
        device: &Device,
        queue: &Queue,
        image: &Bitmap,
        area: Area,
        clips: Clips,
    ) {
        let key = std::ptr::from_ref(image) as usize;

        if !self.pictures.contains_key(&key) {
            let texture = picture::upload(device, queue, image);

            let group = picture_group(device, &self.picture_inputs, &texture, &self.sampler);

            self.pictures.insert(key, group);
        }

        let corners = [0.0, 0.0, 1.0, 1.0];

        self.push(
            area,
            (Corners::default(), 0.0),
            Color::WHITE,
            clips,
            PICTURE,
            corners,
            Some(key),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        area: Area,
        (radius, thickness): (Corners, f32),
        color: Color,
        clips: Clips,
        kind: f32,
        source: [f32; 4],
        picture: Option<usize>,
    ) {
        let outer = clips.outer.unwrap_or(NO_CLIP);
        let inner = clips.inner.unwrap_or(NO_CLIP);

        let channel = |value: u8| f32::from(value) / 255.0;

        let index = (self.waiting.len() / QUAD_SIZE) as u32;

        // the fields of Quad in quads.wgsl, in the same order
        let area_row = [area.x, area.y, area.width, area.height];
        let clip_row = [
            outer.area.x,
            outer.area.y,
            outer.area.width,
            outer.area.height,
        ];
        let color_row = [
            channel(color.r),
            channel(color.g),
            channel(color.b),
            channel(color.a),
        ];
        let shape_row = [thickness, 0.0, 0.0, kind];
        let source_row = source;
        let inner_clip_row = [
            inner.area.x,
            inner.area.y,
            inner.area.width,
            inner.area.height,
        ];
        let radius_row = radius.to_array();
        let clip_radius_row = outer.radius.to_array();
        let inner_radius_row = inner.radius.to_array();

        let rows = [
            area_row,
            clip_row,
            color_row,
            shape_row,
            source_row,
            inner_clip_row,
            radius_row,
            clip_radius_row,
            inner_radius_row,
        ];

        for row in rows {
            self.waiting.extend(row);
        }

        // a quad reading the same image as the one before joins its run
        if let Some(run) = self.runs.last_mut()
            && run.picture == picture
        {
            run.count += 1;

            return;
        }

        self.runs.push(Run {
            picture,
            first: index,
            count: 1,
        });
    }
}
