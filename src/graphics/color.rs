#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub const BLACK: Self = Self::rgb(0, 0, 0);

    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const RED: Self = Self::rgb(255, 0, 0);

    pub const GREEN: Self = Self::rgb(0, 255, 0);

    pub const BLUE: Self = Self::rgb(0, 0, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // "#rrggbb", or "#rrggbbaa" when not fully opaque
    pub fn hex(&self) -> String {
        let rgb = format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b);

        if self.a == 255 {
            return rgb;
        }

        format!("{rgb}{:02x}", self.a)
    }
}

// accepts "#rgb", "#rrggbb" and "#rrggbbaa", the # is optional
impl From<&str> for Color {
    fn from(hex: &str) -> Self {
        let digits = hex.trim_start_matches('#');

        let channels: Vec<u8> = match digits.len() {
            3 => digits.chars().map(short_channel).collect(),
            6 | 8 => digits.as_bytes().chunks(2).map(long_channel).collect(),
            _ => panic!("failed to parse hex color: {hex}"),
        };

        // six digits leave out alpha, which means fully opaque
        let alpha = channels.get(3).copied().unwrap_or(255);

        Self::rgba(channels[0], channels[1], channels[2], alpha)
    }
}

// one digit stands for itself repeated, so "f" means "ff"
fn short_channel(digit: char) -> u8 {
    let value = digit.to_digit(16).expect("failed to parse hex color") as u8;

    value * 17
}

fn long_channel(pair: &[u8]) -> u8 {
    let pair = std::str::from_utf8(pair).expect("failed to parse hex color");

    u8::from_str_radix(pair, 16).expect("failed to parse hex color")
}
