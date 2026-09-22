use crate::graphics::Renderer;

use super::{Direction, Widget};

pub(crate) struct Layout {
    direction: Direction,
    children: Vec<Box<dyn Widget>>,
}

impl Layout {
    pub(crate) fn new(direction: Direction, children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            direction,
            children,
        }
    }
}

impl Widget for Layout {
    fn width(&self) -> f32 {
        match self.direction {
            Direction::Row => self.children.iter().map(|child| child.width()).sum(),

            Direction::Column => self
                .children
                .iter()
                .map(|child| child.width())
                .fold(0.0, f32::max),
        }
    }

    fn height(&self) -> f32 {
        match self.direction {
            Direction::Row => self
                .children
                .iter()
                .map(|child| child.height())
                .fold(0.0, f32::max),

            Direction::Column => self.children.iter().map(|child| child.height()).sum(),
        }
    }

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32) {
        let mut current_x = x;
        let mut current_y = y;

        for child in &self.children {
            child.draw(renderer, current_x, current_y);

            match self.direction {
                Direction::Row => {
                    current_x += child.width();
                }

                Direction::Column => {
                    current_y += child.height();
                }
            }
        }
    }
}
