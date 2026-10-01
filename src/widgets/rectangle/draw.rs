use crate::animation::moving;
use crate::graphics::{Rect, Renderer, image};
use crate::input::Target;
use crate::widgets::Widget;
use crate::{Fill, Image, Size};

use super::{Rectangle, child, input, shadow, time, transform};

impl Widget for Rectangle {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let local = transform::local(self, area);

        renderer.transformed(local, |renderer| draw_in_place(self, renderer, area));
    }

    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        input::collect_targets(self, area, targets);
    }
}

// the drawing as if the rectangle were not rotated, scaled or moved
fn draw_in_place(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect) {
    let radius = rectangle.radius.resolve(area.width, area.height);

    renderer.blur(area, radius, rectangle.blur);

    // the rectangle holding a mask is still drawing into this renderer, so the cut lands in it
    if let Fill::Mask = rectangle.fill {
        renderer.cut(area, radius, rectangle.opacity);
    }

    // collected apart, so a mask inside this rectangle cuts no further than its edge
    let mut group = renderer.layer();

    paint(rectangle, &mut group, area, radius);

    renderer.blend(group, rectangle.opacity);
}

fn paint(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    shadow::drop_shadow(rectangle, renderer, area, radius);

    match &rectangle.fill {
        Fill::Color(color) => renderer.rectangle(area, *color, radius),
        Fill::Gradient(gradient) => renderer.gradient(area, gradient, radius),
        Fill::Image(image) => paint_image(image, renderer, area, radius),

        // the cut was already made in the rectangle holding this one
        Fill::Mask => {}
    }

    if let Some(shader) = &rectangle.shader {
        renderer.shader(area, shader, radius, &rectangle.shader_values);

        // the shader's time moves on, so the window keeps drawing new frames
        if time::reads_time(shader) {
            moving::set();
        }
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
    // still decoding, or unreadable
    let Some(image) = image::load(&fill.path, fill.thumbnail, fill.blur) else {
        return;
    };

    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let placement = fill.fit.place(area, image_width, image_height);

    renderer.image(area, radius, image, placement);
}
