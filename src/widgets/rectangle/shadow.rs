use crate::graphics::{Rect, Renderer};
use crate::widgets::shadow::Kind;

use super::Rectangle;

// drawn before the fill, so the fill covers the part under the rectangle
pub fn drop_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
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
pub fn inner_shadow(rectangle: &Rectangle, renderer: &mut Renderer, area: Rect, radius: f32) {
    let Some(shadow) = &rectangle.shadow else {
        return;
    };

    if shadow.kind != Kind::Inner {
        return;
    }

    let hole = shadow.place(area);

    renderer.inner_shadow(area, hole, radius, shadow.tint(), shadow.blur);
}
