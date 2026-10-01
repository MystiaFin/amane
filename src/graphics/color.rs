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

    // read only, so a theme can mix colors without reaching into the fields
    pub fn red(&self) -> u8 {
        self.r
    }

    pub fn green(&self) -> u8 {
        self.g
    }

    pub fn blue(&self) -> u8 {
        self.b
    }

    pub fn alpha(&self) -> u8 {
        self.a
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

// a bright pink no theme uses, so a mistyped color shows on screen instead of crashing the shell
const MISTYPED: Color = Color::rgb(255, 0, 255);

impl From<&str> for Color {
    fn from(hex: &str) -> Self {
        parse(hex).unwrap_or(MISTYPED)
    }
}

// accepts "#rgb", "#rrggbb" and "#rrggbbaa", the # is optional
fn parse(hex: &str) -> Option<Color> {
    let digits = hex.trim_start_matches('#');

    let channels: Option<Vec<u8>> = match digits.len() {
        3 => digits.chars().map(short_channel).collect(),
        6 | 8 => digits.as_bytes().chunks(2).map(long_channel).collect(),
        _ => None,
    };

    let channels = channels?;

    // six digits leave out alpha, which means fully opaque
    let alpha = channels.get(3).copied().unwrap_or(255);

    Some(Color::rgba(channels[0], channels[1], channels[2], alpha))
}

// one digit stands for itself repeated, so "f" means "ff"
fn short_channel(digit: char) -> Option<u8> {
    let value = digit.to_digit(16)? as u8;

    Some(value * 17)
}

fn long_channel(pair: &[u8]) -> Option<u8> {
    let pair = std::str::from_utf8(pair).ok()?;

    u8::from_str_radix(pair, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_hex_and_marks_mistakes() {
        assert_eq!(Color::from("#1e1e2e"), Color::rgb(0x1e, 0x1e, 0x2e));
        assert_eq!(Color::from("fff"), Color::WHITE);
        assert_eq!(Color::from("#00000080"), Color::rgba(0, 0, 0, 0x80));

        assert_eq!(Color::from("#12345"), MISTYPED);
        assert_eq!(Color::from("#zzzzzz"), MISTYPED);
    }
}
