use crate::{Size, Widget};

use super::Direction;

// how wide the children need the layout to be
pub fn width(direction: Direction, children: &[Box<dyn Widget>], gap: f32) -> Size {
    let mut total = 0.0;
    let mut widest = 0.0;

    for child in children {
        // one child that fills makes the whole layout fill
        let Size::Fixed(width) = child.width() else {
            return Size::Parent;
        };

        total += width;

        widest = f32::max(widest, width);
    }

    match direction {
        // side by side: widths and the gaps between them add up
        Direction::Row => Size::Fixed(total + gaps(children, gap)),

        // stacked: as wide as the widest child
        Direction::Column => Size::Fixed(widest),
    }
}

// how tall the children need the layout to be
pub fn height(direction: Direction, children: &[Box<dyn Widget>], gap: f32) -> Size {
    let mut total = 0.0;
    let mut tallest = 0.0;

    for child in children {
        // one child that fills makes the whole layout fill
        let Size::Fixed(height) = child.height() else {
            return Size::Parent;
        };

        total += height;

        tallest = f32::max(tallest, height);
    }

    match direction {
        Direction::Row => Size::Fixed(tallest),
        Direction::Column => Size::Fixed(total + gaps(children, gap)),
    }
}

// all the gaps between children added up
pub fn gaps(children: &[Box<dyn Widget>], gap: f32) -> f32 {
    let count = children.len().saturating_sub(1);

    gap * count as f32
}
