use crate::graphics::{Rect, Renderer, image};
use crate::widgets::Widget;
use crate::widgets::shadow::Kind;
use crate::{Fill, Image, Size};

use super::Rectangle;

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

        if self.opacity == 1.0 {
            paint(self, renderer, area, radius);

            return;
        }

        let mut layer = renderer.layer();

        paint(self, &mut layer, area, radius);

        renderer.blend(layer, self.opacity);
    }
}

fn paint(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    drop_shadow(rectangle, renderer, area, radius);

    match &rectangle.fill {
        Fill::Color(color) => renderer.rectangle(area, *color, radius),
        Fill::Image(image) => paint_image(image, renderer, area, radius),
    }

    inner_shadow(rectangle, renderer, area, radius);

    renderer.border(
        area,
        radius,
        rectangle.border_thickness,
        rectangle.border_color,
    );

    let Some(child) = &rectangle.child else {
        return;
    };

    let child_area = Rect::new(
        area.x,
        area.y,
        child.width().resolve(area.width),
        child.height().resolve(area.height),
    );

    child.draw(renderer, child_area);
}

fn paint_image(fill: &Image, renderer: &mut Renderer, area: Rect, radius: f32) {
    let image = image::load(&fill.path);

    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let placement = fill.fit.place(area, image_width, image_height);

    renderer.image(area, radius, image, placement);
}

// drawn before the fill, so the fill covers the part under the rectangle
fn drop_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    let Some(shadow) = &rectangle.shadow else {
        return;
    };

    if shadow.kind != Kind::Drop {
        return;
    }

    let shadow_area = shadow.place(area);

    renderer.shadow(shadow_area, radius, shadow.tint(), shadow.blur);
}

// drawn over the fill but under the border and child
fn inner_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    let Some(shadow) = &rectangle.shadow else {
        return;
    };

    if shadow.kind != Kind::Inner {
        return;
    }

    let hole = shadow.place(area);

    renderer.inner_shadow(area, hole, radius, shadow.tint(), shadow.blur);
}
