use crate::graphics::{Cap, Color};

pub struct Style {
    pub(crate) fill: Color,

    pub(crate) stroke_thickness: f32,
    pub(crate) stroke_color: Color,
    pub(crate) cap: Cap,

    pub(crate) opacity: f32,
}

// a new shape draws nothing until it gets a fill or a stroke
impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Color::TRANSPARENT,

            stroke_thickness: 0.0,
            stroke_color: Color::TRANSPARENT,
            cap: Cap::Butt,

            opacity: 1.0,
        }
    }
}
