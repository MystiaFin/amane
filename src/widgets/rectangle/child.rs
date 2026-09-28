use crate::graphics::Rect;
use crate::widgets::Widget;

// a child starts at the rectangle's top left corner
pub fn area(child: &dyn Widget, area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y,
        child.width().resolve(area.width),
        child.height().resolve(area.height),
    )
}
