use vello::Scene;
use vello::kurbo::{Affine, Point, Shape};
use vello::peniko::{Fill, ImageAlphaType, ImageBrush, ImageData, ImageQuality};
use vello::wgpu::{Extent3d, Origin3d, TexelCopyTextureInfo, Texture, TextureAspect};

use crate::graphics::{Path, Transform};

use super::paint::{affine, bezier};
use super::{Gpu, pass};

impl Gpu {
    /*
     * blurs a copy of what the canvas holds inside the path,
     * then adds it to the scene cut to the path, and hands back the copy vello now reads from
     */
    pub(super) fn blur(
        &mut self,
        canvas: &Texture,
        scene: &mut Scene,
        path: &Path,
        transform: Transform,
        amount: f32,
    ) -> Option<ImageData> {
        let transform = affine(transform);

        let shape = transform * bezier(path);

        let bounds = shape.bounding_box();

        // a rectangle hanging past the window edge only blurs the part inside
        let left = f64::max(bounds.x0.floor(), 0.0);
        let top = f64::max(bounds.y0.floor(), 0.0);
        let right = f64::min(bounds.x1.ceil(), f64::from(canvas.width()));
        let bottom = f64::min(bounds.y1.ceil(), f64::from(canvas.height()));

        if right <= left || bottom <= top {
            return None;
        }

        let x = left as u32;
        let y = top as u32;
        let width = (right - left) as u32;
        let height = (bottom - top) as u32;

        // the amount is in logical pixels, the copy is in real ones
        let amount = f64::from(amount);

        let reach = transform * Point::new(amount, amount) - transform * Point::ZERO;

        let first = pass::blur(&self.device, width, height);
        let second = pass::blur(&self.device, width, height);

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let area = TexelCopyTextureInfo {
            texture: canvas,
            mip_level: 0,
            origin: Origin3d { x, y, z: 0 },
            aspect: TextureAspect::All,
        };

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        encoder.copy_texture_to_texture(area, first.as_image_copy(), size);

        let first_view = pass::view(&first);
        let second_view = pass::view(&second);

        let horizontal_reach = reach.x.round() as f32;
        let vertical_reach = reach.y.round() as f32;

        // three box blurs in a row look close to a gaussian blur
        for _ in 0..3 {
            let along_row = [1.0, 0.0, horizontal_reach, 0.0];
            let along_column = [0.0, 1.0, vertical_reach, 0.0];

            self.box_blur.run(
                &self.device,
                &mut encoder,
                &first_view,
                &second_view,
                along_row,
            );

            self.box_blur.run(
                &self.device,
                &mut encoder,
                &second_view,
                &first_view,
                along_column,
            );
        }

        self.queue.submit([encoder.finish()]);

        let mut blurred = self.vello.register_texture(first);

        // the copy came from the canvas, which keeps its colors premultiplied
        blurred.alpha_type = ImageAlphaType::AlphaPremultiplied;

        // the copy lines up with real pixels, so no smoothing is needed
        let brush = ImageBrush::new(blurred.clone()).with_quality(ImageQuality::Low);

        let position = Affine::translate((left, top));

        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &shape);

        scene.draw_image(&brush, position);

        scene.pop_layer();

        Some(blurred)
    }
}
