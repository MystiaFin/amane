use std::rc::Rc;

use crate::{Button, Key, Scroll};

/*
 * shared, so the hit areas kept after a redraw
 * still work once the widget tree they came from is dropped
 */
pub type ClickHandler = Rc<dyn Fn(Button)>;
pub type HoverHandler = Rc<dyn Fn(bool)>;
pub type ScrollHandler = Rc<dyn Fn(Scroll)>;
pub type KeyHandler = Rc<dyn Fn(Key)>;

#[derive(Clone, Default)]
pub struct Handlers {
    pub click: Option<ClickHandler>,
    pub hover: Option<HoverHandler>,
    pub scroll: Option<ScrollHandler>,
}
