use std::rc::Rc;

use crate::graphics::Rect;
use crate::input::Target;
use crate::{Button, Rectangle, Scroll};

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

    let child_area = child::area(child.as_ref(), area);

    child.collect_targets(child_area, targets);
}
