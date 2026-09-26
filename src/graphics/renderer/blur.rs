use tiny_skia::{FillRule, IntRect, Mask, Pixmap, PixmapPaint, Point, Transform};

use crate::graphics::{Rect, Renderer};

impl Renderer {
    pub fn blur(&mut self, rect: Rect, radius: f32, amount: f32) {
        // most rectangles ask for no blur, so they skip copying pixels
        if amount == 0.0 {
            return;
        }

        let Some(path) = rect.trace(radius) else {
            return;
        };

        let Some(real_bounds) = path.bounds().transform(self.transform) else {
            return;
        };

        let Some(real_area) = real_bounds.round_out() else {
            return;
        };

        let pixmap_area = IntRect::from_xywh(0, 0, self.pixmap.width(), self.pixmap.height())
            .expect("failed to measure pixmap");

        // a rectangle hanging past the window edge only blurs the part inside
        let Some(area) = real_area.intersect(&pixmap_area) else {
            return;
        };

        let Some(mut behind) = self.pixmap.clone_rect(area) else {
            return;
        };

        // the amount is in logical pixels, the copy is in real ones
        let mut reach = Point::from_xy(amount, amount);

        self.transform.map_point(&mut reach);

        let horizontal_reach = reach.x.round() as i32;
        let vertical_reach = reach.y.round() as i32;

        // stepping one pixel right walks along a row, one pixel down walks along a column
        let row_step_x = 1;
        let row_step_y = 0;
        let column_step_x = 0;
        let column_step_y = 1;

        // three box blurs in a row look close to a gaussian blur
        for _ in 0..3 {
            spread(&mut behind, row_step_x, row_step_y, horizontal_reach);
            spread(&mut behind, column_step_x, column_step_y, vertical_reach);
        }

        let mut shape =
            Mask::new(self.pixmap.width(), self.pixmap.height()).expect("failed to create mask");

        shape.fill_path(&path, FillRule::Winding, true, self.transform);

        let paint = PixmapPaint::default();

        // the copy is already in real pixels, so it goes back unscaled
        self.pixmap.draw_pixmap(
            area.x(),
            area.y(),
            behind.as_ref(),
            &paint,
            Transform::identity(),
            Some(&shape),
        );
    }
}

/*
 * averages each pixel with its neighbours up to reach pixels away,
 * walking one step_x, step_y at a time, so one call blurs along one direction
 */
fn spread(pixmap: &mut Pixmap, step_x: i32, step_y: i32, reach: i32) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    let original = pixmap.data().to_vec();

    let pixels = pixmap.data_mut();

    let neighbours = (reach * 2 + 1) as u32;

    for y in 0..height {
        for x in 0..width {
            let mut total = [0; 4];

            for offset in -reach..=reach {
                // past the edge, the edge pixel repeats
                let neighbour_x = i32::clamp(x + offset * step_x, 0, width - 1);
                let neighbour_y = i32::clamp(y + offset * step_y, 0, height - 1);

                let neighbour = ((neighbour_y * width + neighbour_x) * 4) as usize;

                for channel in 0..4 {
                    total[channel] += u32::from(original[neighbour + channel]);
                }
            }

            let pixel = ((y * width + x) * 4) as usize;

            for channel in 0..4 {
                pixels[pixel + channel] = (total[channel] / neighbours) as u8;
            }
        }
    }
}
