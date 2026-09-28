#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Urgency {
    Low,

    // what a notification without the hint gets
    #[default]
    Normal,

    Critical,
}

impl Urgency {
    // the spec sends urgency as a byte: 0 low, 1 normal, 2 critical
    pub(crate) fn from_level(level: f64) -> Self {
        match level as u8 {
            0 => Self::Low,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }
}
