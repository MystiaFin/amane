use super::{Center, End, Start};

// where a row or column puts each child across the direction it grows in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,

    Center,

    End,
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
