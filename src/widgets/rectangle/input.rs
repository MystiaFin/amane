use std::rc::Rc;

use crate::graphics::Rect;
use crate::input::{Target, clip};
use crate::{Button, Point, Rectangle, Scroll};

use super::child;

impl Rectangle {
    pub fn on_click(mut self, handler: impl Fn(Button) + 'static) -> Self {
        self.handlers.click = Some(Rc::new(handler));

        self
    }

    // true when the pointer comes in, false when it goes out
    pub fn on_hover(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.handlers.hover = Some(Rc::new(handler));

        self
    }

    pub fn on_scroll(mut self, handler: impl Fn(Scroll) + 'static) -> Self {
        self.handlers.scroll = Some(Rc::new(handler));

        self
    }

    /*
     * runs on a left press and on every move until the release,
     * even after the pointer leaves the rectangle
     */
    pub fn on_drag(mut self, handler: impl Fn(Point) + 'static) -> Self {
        self.handlers.drag = Some(Rc::new(handler));

        self
    }

    // runs on every move while the pointer is over the rectangle
    pub fn on_move(mut self, handler: impl Fn(Point) + 'static) -> Self {
        self.handlers.motion = Some(Rc::new(handler));

        self
    }
}

pub fn collect_targets(rectangle: &Rectangle, area: Rect, targets: &mut Vec<Target>) {
    let target = Target {
        area,
        handlers: rectangle.handlers.clone(),
    };

    // added before the child, so the child wins where both react to the same thing
    targets.push(target);

    let Some(child) = &rectangle.child else {
        return;
    };

    let child_area = child::area(rectangle, child.as_ref(), area);

    let first = targets.len();

    child.collect_targets(child_area, targets);

    // the rounded corners are left out, they are too small to miss a click by
    if rectangle.clip {
        clip(&mut targets[first..], area);
    }
}
