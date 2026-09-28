#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl Weight {
    // the style name fontconfig looks a font file up by
    pub(crate) fn style(self) -> &'static str {
        match self {
            Weight::Thin => "Thin",
            Weight::ExtraLight => "ExtraLight",
            Weight::Light => "Light",
            Weight::Regular => "Regular",
            Weight::Medium => "Medium",
            Weight::SemiBold => "SemiBold",
            Weight::Bold => "Bold",
            Weight::ExtraBold => "ExtraBold",
            Weight::Black => "Black",
        }
    }
}

impl From<u16> for Weight {
    // font files only come in named weights, so a number picks the closest one
    fn from(number: u16) -> Self {
        let hundreds = number.saturating_add(50) / 100;

        match hundreds {
            0 | 1 => Weight::Thin,
            2 => Weight::ExtraLight,
            3 => Weight::Light,
            4 => Weight::Regular,
            5 => Weight::Medium,
            6 => Weight::SemiBold,
            7 => Weight::Bold,
            8 => Weight::ExtraBold,
            _ => Weight::Black,
        }
    }
}
