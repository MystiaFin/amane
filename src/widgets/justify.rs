use super::{Center, End, Start};

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
