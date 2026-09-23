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
