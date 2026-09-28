use crate::graphics::{Rect, Renderer, image};
use crate::input::Target;
use crate::widgets::Widget;
use crate::{Fill, Image, Size};

use super::{Rectangle, child, input, shadow};

impl Widget for Rectangle {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let radius = self.radius.resolve(area.width, area.height);

        renderer.blur(area, radius, self.blur);

        // the rectangle holding a mask is still drawing into this renderer, so the cut lands in it
        if let Fill::Mask = self.fill {
            renderer.cut(area, radius, self.opacity);
        }

        // collected apart, so a mask inside this rectangle cuts no further than its edge
        let mut group = renderer.layer();

        paint(self, &mut group, area, radius);

        renderer.blend(group, self.opacity);
    }

    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        input::collect_targets(self, area, targets);
    }
}

fn paint(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    shadow::drop_shadow(rectangle, renderer, area, radius);

    match &rectangle.fill {
        Fill::Color(color) => renderer.rectangle(area, *color, radius),
        Fill::Image(image) => paint_image(image, renderer, area, radius),

        // the cut was already made in the rectangle holding this one
        Fill::Mask => {}
    }

    shadow::inner_shadow(rectangle, renderer, area, radius);

    renderer.border(
        area,
        radius,
        rectangle.border_thickness,
        rectangle.border_color,
    );

    let Some(child) = &rectangle.child else {
        return;
    };

    let child_area = child::area(rectangle, child.as_ref(), area);

    if !rectangle.clip {
        child.draw(renderer, child_area);

        return;
    }

    let mut inside = renderer.layer();

    child.draw(&mut inside, child_area);

    renderer.clip(inside, area, radius);
}

fn paint_image(fill: &Image, renderer: &mut Renderer, area: Rect, radius: f32) {
    let image = image::load(&fill.path);

    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let placement = fill.fit.place(area, image_width, image_height);

    renderer.image(area, radius, image, placement);
}
