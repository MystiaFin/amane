use crate::graphics::Rect;

// bare names for align and justify, so a view can say .justify(Center)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Start;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Center;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct End;

// where a row or column puts each child across the direction it grows in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,

    Center,

    End,
}

// how a row or column spreads its children along the direction it grows in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,

    Center,

    End,

    // first and last child touch the edges, the rest of the space goes between them
    SpaceBetween,

    // every child gets the same space on both sides, so the edges get half a gap
    SpaceAround,

    // the edges and every gap between children get the same space
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Parent,
    Fixed(f32),
}

// empty space kept between a rectangle's edges and its child
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Align {
    // how far a child moves in from the start edge, given the room it leaves
    pub(crate) fn offset(self, free: f32) -> f32 {
        match self {
            Align::Start => 0.0,
            Align::Center => free / 2.0,
            Align::End => free,
        }
    }
}

impl From<Start> for Align {
    fn from(_: Start) -> Self {
        Self::Start
    }
}

impl From<Center> for Align {
    fn from(_: Center) -> Self {
        Self::Center
    }
}

impl From<End> for Align {
    fn from(_: End) -> Self {
        Self::End
    }
}

impl Justify {
    // the space before the first child and the space between each pair
    pub(crate) fn spread(self, free: f32, count: usize) -> (f32, f32) {
        let count = count as f32;

        match self {
            Justify::Start => (0.0, 0.0),
            Justify::Center => (free / 2.0, 0.0),
            Justify::End => (free, 0.0),
            Justify::SpaceBetween => Self::between(free, count),

            Justify::SpaceAround => {
                let gap = free / count;

                (gap / 2.0, gap)
            }

            Justify::SpaceEvenly => {
                let gap = free / (count + 1.0);

                (gap, gap)
            }
        }
    }

    // a single child has nothing to sit between, so it stays at the start
    fn between(free: f32, count: f32) -> (f32, f32) {
        if count < 2.0 {
            return (0.0, 0.0);
        }

        (0.0, free / (count - 1.0))
    }
}

impl From<Start> for Justify {
    fn from(_: Start) -> Self {
        Self::Start
    }
}

impl From<Center> for Justify {
    fn from(_: Center) -> Self {
        Self::Center
    }
}

impl From<End> for Justify {
    fn from(_: End) -> Self {
        Self::End
    }
}

impl Size {
    pub(crate) fn resolve(self, available: f32) -> f32 {
        match self {
            Size::Parent => available,
            Size::Fixed(pixels) => pixels,
        }
    }
}

impl From<f32> for Size {
    fn from(pixels: f32) -> Self {
        Self::Fixed(pixels)
    }
}

impl Padding {
    // the area left once the padding is taken off every side
    pub(crate) fn shrink(self, area: Rect) -> Rect {
        let width = f32::max(area.width - self.left - self.right, 0.0);
        let height = f32::max(area.height - self.top - self.bottom, 0.0);

        Rect::new(area.x + self.left, area.y + self.top, width, height)
    }
}

// the same space on every side
impl From<f32> for Padding {
    fn from(pixels: f32) -> Self {
        Self {
            top: pixels,
            right: pixels,
            bottom: pixels,
            left: pixels,
        }
    }
}
