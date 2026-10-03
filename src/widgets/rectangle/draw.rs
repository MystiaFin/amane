use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::animation::moving;
use crate::graphics::{Area, Renderer, image};
use crate::input::Target;
use crate::style::Kind;
use crate::{Fill, Image, Size, Widget};

use super::{Rectangle, child, targets, transform};

thread_local! {
    // read once per shader file, a shader's source doesn't change while the shell runs
    static READS_TIME: RefCell<HashMap<PathBuf, bool>> = RefCell::new(HashMap::new());
}

impl Widget for Rectangle {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Area) {
        let local = transform::local(self, area);

        renderer.transformed(local, |renderer| draw_in_place(self, renderer, area));
    }

    fn collect_targets(&self, area: Area, targets: &mut Vec<Target>) {
        targets::collect_targets(self, area, targets);
    }
}

// the drawing as if the rectangle were not rotated, scaled or moved
fn draw_in_place(rectangle: &Rectangle, renderer: &mut Renderer, area: Area) {
    let radius = rectangle.radius.resolve(area.width, area.height);

    renderer.blur(area, radius, rectangle.blur);

    // the rectangle holding a mask is still drawing into this renderer, so the cut lands in it
    if let Fill::Mask = rectangle.fill {
        renderer.cut(area, radius, rectangle.opacity);
    }

    // collected apart, so a mask inside this rectangle cuts no further than its edge
    let mut group = renderer.group();

    paint(rectangle, &mut group, area, radius);

    renderer.blend(group, rectangle.opacity);
}

fn paint(rectangle: &Rectangle, renderer: &mut Renderer, area: Area, radius: f32) {
    drop_shadow(rectangle, renderer, area, radius);

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
        if reads_time(shader) {
            moving::set();
        }
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

    let child_area = child::area(rectangle, child.as_ref(), area);

    if !rectangle.clip {
        child.draw(renderer, child_area);

        return;
    }

    let mut inside = renderer.group();

    child.draw(&mut inside, child_area);

    renderer.clip(inside, area, radius);
}

fn paint_image(fill: &Image, renderer: &mut Renderer, area: Area, radius: f32) {
    // still decoding, or unreadable
    let Some(image) = image::load(&fill.path, fill.thumbnail, fill.blur) else {
        return;
    };

    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let placement = fill.fit.place(area, image_width, image_height);

    renderer.image(area, radius, image, placement);
}

// drawn before the fill, so the fill covers the part under the rectangle
pub fn drop_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Area, radius: f32) {
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
pub fn inner_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Area, radius: f32) {
    let Some(shadow) = &rectangle.shadow else {
        return;
    };

    if shadow.kind != Kind::Inner {
        return;
    }

    let hole = shadow.place(area);

    renderer.inner_shadow(area, hole, radius, shadow.tint(), shadow.blur);
}

/*
 * only a shader that reads time changes between frames, so only that
 * one keeps the window drawing; any `time` word counts, even in a comment
 */
pub fn reads_time(shader: &Path) -> bool {
    READS_TIME.with_borrow_mut(|known| {
        if let Some(&reads) = known.get(shader) {
            return reads;
        }

        let source = fs::read_to_string(shader).unwrap_or_default();

        let is_name_part = |character: char| character.is_alphanumeric() || character == '_';

        let reads = source.split(|character| !is_name_part(character)).any(|word| word == "time");

        known.insert(shader.to_path_buf(), reads);

        reads
    })
}
