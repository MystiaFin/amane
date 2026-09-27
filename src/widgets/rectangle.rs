mod needs_height;
mod needs_width;

use crate::graphics::{Color, Rect, Renderer, image};
use crate::{Fill, Image, Radius, Size};

use super::Widget;

pub use needs_height::NeedsHeight;
pub use needs_width::NeedsWidth;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) fill: Fill,
    pub(crate) radius: Radius,
    pub(crate) border_thickness: f32,
    pub(crate) border_color: Color,
    pub(crate) blur: f32,
    pub(crate) opacity: f32,
    pub(crate) child: Option<Box<dyn Widget>>,
}

impl Rectangle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    pub fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();

        self
    }

    pub fn radius(mut self, radius: impl Into<Radius>) -> Self {
        self.radius = radius.into();

        self
    }

    pub fn border(mut self, thickness: f32, color: impl Into<Color>) -> Self {
        self.border_thickness = thickness;
        self.border_color = color.into();

        self
    }

    pub fn blur(mut self, amount: f32) -> Self {
        self.blur = amount;

        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));

        self
    }
}

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
    match &rectangle.fill {
        Fill::Color(color) => renderer.rectangle(area, *color, radius),
        Fill::Image(image) => paint_image(image, renderer, area, radius),
    }

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
