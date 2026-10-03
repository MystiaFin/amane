use crate::graphics::Area;
use crate::Widget;

use super::Rectangle;

// a child sits inside the padding, placed by the rectangle's child alignment
pub fn area(rectangle: &Rectangle, child: &dyn Widget, area: Area) -> Area {
    let inside = rectangle.padding.shrink(area);

    let width = child.width().resolve(inside.width);
    let height = child.height().resolve(inside.height);

    let x = inside.x + rectangle.child_horizontal.offset(inside.width - width);
    let y = inside.y + rectangle.child_vertical.offset(inside.height - height);

    Area::new(x, y, width, height)
}
