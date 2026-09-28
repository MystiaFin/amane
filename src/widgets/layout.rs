mod measure;

use crate::graphics::{Rect, Renderer};
use crate::input::Target;
use crate::{Align, Justify, Size};

use super::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
}

pub struct Layout {
    direction: Direction,
    children: Vec<Box<dyn Widget>>,

    pub(crate) width: Size,
    pub(crate) height: Size,

    pub(crate) justify: Justify,
    pub(crate) align: Align,
}

impl Layout {
    pub fn new(direction: Direction, children: Vec<Box<dyn Widget>>) -> Self {
        // children never change after this, so their size can be worked out once
        let width = measure::width(direction, &children);
        let height = measure::height(direction, &children);

        Self {
            direction,
            children,
            width,
            height,
            justify: Justify::default(),
            align: Align::default(),
        }
    }

    // the size of a child along the direction the layout grows in
    fn along(&self, child: &dyn Widget) -> Size {
        match self.direction {
            Direction::Row => child.width(),
            Direction::Column => child.height(),
        }
    }

    // the size of a child across the direction the layout grows in
    fn across(&self, child: &dyn Widget) -> Size {
        match self.direction {
            Direction::Row => child.height(),
            Direction::Column => child.width(),
        }
    }

    // the area's length along the direction, then its length across it
    fn span(&self, area: Rect) -> (f32, f32) {
        match self.direction {
            Direction::Row => (area.width, area.height),
            Direction::Column => (area.height, area.width),
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

        let (available, _) = self.span(area);

        f32::max(available - used, 0.0) / filling as f32
    }

    // where each child goes, one after another along the direction
    fn place(&self, area: Rect) -> Vec<Rect> {
        let share = self.share(area);
        let (available, room) = self.span(area);

        let mut sizes = Vec::new();
        let mut used = 0.0;

        for child in &self.children {
            let length = self.along(child.as_ref()).resolve(share);
            let thickness = self.across(child.as_ref()).resolve(room);

            used += length;

            sizes.push((length, thickness));
        }

        // Parent-sized children already took the free space, so this is 0 when there are any
        let free = f32::max(available - used, 0.0);
        let (lead, gap) = self.justify.spread(free, self.children.len());

        let mut current = lead;
        let mut child_areas = Vec::new();

        for (length, thickness) in sizes {
            let offset = self.align.offset(room - thickness);

            let (x, y, width, height) = match self.direction {
                Direction::Row => (area.x + current, area.y + offset, length, thickness),
                Direction::Column => (area.x + offset, area.y + current, thickness, length),
            };

            child_areas.push(Rect::new(x, y, width, height));

            current += length + gap;
        }

        child_areas
    }
}

impl Widget for Layout {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let child_areas = self.place(area);

        for (child, child_area) in self.children.iter().zip(child_areas) {
            child.draw(renderer, child_area);
        }
    }

    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        let child_areas = self.place(area);

        for (child, child_area) in self.children.iter().zip(child_areas) {
            child.collect_targets(child_area, targets);
        }
    }
}
