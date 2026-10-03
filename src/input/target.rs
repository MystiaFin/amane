use std::rc::Rc;

use crate::graphics::{Rect, Transform};
use crate::{Button, Cursor, Key, Point, Scroll};

/*
 * shared, so the hit areas kept after a redraw
 * still work once the widget tree they came from is dropped
 */
pub type ClickHandler = Rc<dyn Fn(Button)>;
pub type HoverHandler = Rc<dyn Fn(bool)>;
pub type ScrollHandler = Rc<dyn Fn(Scroll)>;
pub type KeyHandler = Rc<dyn Fn(Key)>;
pub type PointHandler = Rc<dyn Fn(Point)>;

#[derive(Clone, Default)]
pub struct Handlers {
    pub click: Option<ClickHandler>,
    pub hover: Option<HoverHandler>,
    pub scroll: Option<ScrollHandler>,
    pub drag: Option<PointHandler>,
    pub motion: Option<PointHandler>,

    // not a handler, but it belongs to the same spot the pointer is over
    pub cursor: Option<Cursor>,
}

// where a widget was drawn, and what it does when the pointer uses that spot
pub struct Target {
    pub area: Rect,
    pub handlers: Handlers,

    // turns a window position into the area's own space, undoing rotated, scaled or moved parents
    pub inverse: Transform,
}

impl Target {
    pub fn new(area: Rect, handlers: Handlers) -> Self {
        Self {
            area,
            handlers,
            inverse: Transform::IDENTITY,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        let (local_x, local_y) = self.inverse.map(x, y);

        self.area.contains(local_x, local_y)
    }

    // the position measured from the area's top-left corner
    pub fn point(&self, x: f32, y: f32) -> Point {
        let (local_x, local_y) = self.inverse.map(x, y);

        Point {
            x: local_x - self.area.x,
            y: local_y - self.area.y,
        }
    }
}

/*
 * a target that is clipped only reacts where it still shows,
 * the ones fully clipped out stay in the list so positions don't shift
 */
pub fn clip(targets: &mut [Target], area: Rect) {
    for target in targets {
        target.area = target.area.intersect(area);
    }
}
