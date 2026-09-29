// how the speed changes between the start and the target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    // the same speed the whole way
    Linear,

    // starts fast and slows down as it arrives
    #[default]
    Out,

    // starts slow, speeds up in the middle and slows down again
    InOut,
}

impl Easing {
    // turns how much time has passed into how far along the way it is, both from 0 to 1
    pub(crate) fn apply(self, progress: f32) -> f32 {
        match self {
            Easing::Linear => progress,
            Easing::Out => Self::out(progress),
            Easing::InOut => Self::in_out(progress),
        }
    }

    fn out(progress: f32) -> f32 {
        let remaining = 1.0 - progress;

        1.0 - remaining * remaining * remaining
    }

    fn in_out(progress: f32) -> f32 {
        if progress < 0.5 {
            return 4.0 * progress * progress * progress;
        }

        let remaining = 2.0 - 2.0 * progress;

        1.0 - remaining * remaining * remaining / 2.0
    }
}
