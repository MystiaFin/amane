use crate::Size;
use crate::graphics::{Rect, Renderer};

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

    // the size of a child along the direction the layout grows in
    fn along(&self, child: &dyn Widget) -> Size {
        match self.direction {
            Direction::Row => child.width(),
            Direction::Column => child.height(),
        }
    }

    // the space each Parent-sized child gets: what the fixed children leave, split evenly
    fn share(&self, area: Rect) -> f32 {
        let mut used = 0.0;
        let mut filling = 0;

        for child in &self.children {
            match self.along(child.as_ref()) {
                Size::Fixed(pixels) => used += pixels,
                Size::Parent => filling += 1,
            }
        }

        if filling == 0 {
            return 0.0;
        }

        let available = match self.direction {
            Direction::Row => area.width,
            Direction::Column => area.height,
        };

        f32::max(available - used, 0.0) / filling as f32
    }
}

impl Widget for Layout {
    fn width(&self) -> Size {
        let mut total = 0.0;
        let mut widest = 0.0;

        for child in &self.children {
            // one child that fills makes the whole layout fill
            let Size::Fixed(width) = child.width() else {
                return Size::Parent;
            };

            total += width;

            widest = f32::max(widest, width);
        }

        match self.direction {
            // side by side: widths add up
            Direction::Row => Size::Fixed(total),

            // stacked: as wide as the widest child
            Direction::Column => Size::Fixed(widest),
        }
    }

    fn height(&self) -> Size {
        let mut total = 0.0;
        let mut tallest = 0.0;

        for child in &self.children {
            // one child that fills makes the whole layout fill
            let Size::Fixed(height) = child.height() else {
                return Size::Parent;
            };

            total += height;

            tallest = f32::max(tallest, height);
        }

        match self.direction {
            Direction::Row => Size::Fixed(tallest),
            Direction::Column => Size::Fixed(total),
        }
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let share = self.share(area);

        let mut current_x = area.x;
        let mut current_y = area.y;

        for child in &self.children {
            let (width, height) = match self.direction {
                Direction::Row => (
                    child.width().resolve(share),
                    child.height().resolve(area.height),
                ),

                Direction::Column => (
                    child.width().resolve(area.width),
                    child.height().resolve(share),
                ),
            };

            child.draw(renderer, Rect::new(current_x, current_y, width, height));

            match self.direction {
                Direction::Row => {
                    current_x += width;
                }

                Direction::Column => {
                    current_y += height;
                }
            }
        }
    }
}
