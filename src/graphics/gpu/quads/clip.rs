use crate::graphics::{Area, Corners};

// nothing clips: far bigger than any canvas
pub const NO_CLIP: Clip = Clip {
    area: Area::new(-1.0e6, -1.0e6, 2.0e6, 2.0e6),
    radius: Corners {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    },
};

// a rounded rectangle on the canvas that quads only show inside
#[derive(Clone, Copy)]
pub struct Clip {
    pub area: Area,
    pub radius: Corners,
}

// at most two rounded clips reach a quad, the outer one and the one inside it
#[derive(Clone, Copy, Default)]
pub struct Clips {
    pub outer: Option<Clip>,
    pub inner: Option<Clip>,
}

impl Clips {
    // a clip inside the ones already there; with two there already, the outer two merge
    pub fn within(self, clip: Clip) -> Clips {
        match (self.outer, self.inner) {
            (None, _) => Clips {
                outer: Some(clip),
                inner: None,
            },

            (Some(outer), None) => Clips {
                outer: Some(outer),
                inner: Some(clip),
            },

            (Some(outer), Some(inner)) => Clips {
                outer: Some(narrow(outer, inner)),
                inner: Some(clip),
            },
        }
    }
}

/*
 * only two rounded clips reach a quad, so deeper ones merge into their
 * overlap, rounded like the inner one; clips inside clips are almost
 * always smaller, so this is close enough for them
 */
fn narrow(outer: Clip, inner: Clip) -> Clip {
    Clip {
        area: outer.area.intersect(inner.area),
        radius: inner.radius,
    }
}
