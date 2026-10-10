use crate::Color;

// colors that blend into each other, each stop is a spot from 0 to 1 and its color
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    // 0 degrees runs bottom to top, 90 runs left to right
    Linear {
        angle: f32,
        stops: Vec<(f32, Color)>,
    },

    // from the center out to the corners
    Radial {
        stops: Vec<(f32, Color)>,
    },
}

impl Gradient {
    pub fn linear<C: Into<Color>>(angle: f32, stops: impl IntoIterator<Item = (f32, C)>) -> Self {
        Self::Linear {
            angle,
            stops: collect(stops),
        }
    }

    pub fn radial<C: Into<Color>>(stops: impl IntoIterator<Item = (f32, C)>) -> Self {
        Self::Radial {
            stops: collect(stops),
        }
    }
}

// the stops can be plain hex strings like "#1e1e2e" as well as colors
fn collect<C: Into<Color>>(stops: impl IntoIterator<Item = (f32, C)>) -> Vec<(f32, Color)> {
    let mut collected = Vec::new();

    for (offset, color) in stops {
        collected.push((offset, color.into()));
    }

    collected
}
