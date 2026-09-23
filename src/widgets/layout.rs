use crate::graphics::Renderer;

use super::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
}

pub struct Layout {
    direction: Direction,
    children: Vec<Box<dyn Widget>>,
}

impl Layout {
    pub fn new(direction: Direction, children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            direction,
            children,
        }
    }
}

impl Widget for Layout {
    fn width(&self) -> f32 {
        let mut total = 0.0;
        let mut widest = 0.0;

        for child in &self.children {
            total += child.width();

            widest = f32::max(widest, child.width());
        }

        match self.direction {
            // side by side: widths add up
            Direction::Row => total,

            // stacked: as wide as the widest child
            Direction::Column => widest,
        }
    }

    fn height(&self) -> f32 {
        let mut total = 0.0;
        let mut tallest = 0.0;

        for child in &self.children {
            total += child.height();

            tallest = f32::max(tallest, child.height());
        }

        match self.direction {
            // side by side: as tall as the tallest child
            Direction::Row => tallest,

            // stacked: heights add up
            Direction::Column => total,
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
