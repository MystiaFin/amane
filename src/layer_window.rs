use create::ui::{Direction, Widget};

pub struct LayerWindow<W> {
    pub width: u32,
    pub height: u32,

    pub(crate) direction: Direction,
    pub(crate) children: Vec<Box<dyn Widget>>,
}

impl LayerWindow {
    pub fn new(width: u32, height: u32, children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            width,
            height,
            direction: Direction::Row,
            children,
        }
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}
