use std::rc::Rc;

use crate::{Button, Key, Point, Scroll};

use super::Cursor;

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
