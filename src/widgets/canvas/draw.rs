use crate::graphics::{Rect, Renderer};
use crate::widgets::Widget;
use crate::{Color, Shape, Size};

use super::Canvas;

impl Widget for Canvas {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        for shape in &self.shapes {
            paint(shape.as_ref(), renderer, area);
        }
    }
}

fn paint(shape: &dyn Shape, renderer: &mut Renderer, area: Rect) {
    let Some(path) = shape.trace(area.width, area.height) else {
        return;
    };

    let style = shape.style();

    // fill and line are faded together, so the line doesn't show the fill through it
    let mut group = renderer.layer();

    // shapes start without a fill, and drawing an invisible one would still cost the gpu
    if style.fill != Color::TRANSPARENT {
        group.fill_path(&path, area, style.fill);
    }

    // a zero width line would still draw as a hairline
    if style.stroke_thickness > 0.0 {
        group.stroke_path(
            &path,
            area,
            style.stroke_thickness,
            style.stroke_color,
            style.cap,
        );
    }

    renderer.blend(group, style.opacity);
}
