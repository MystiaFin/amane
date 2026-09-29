// how the speed changes between the start and the target
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    // the same speed the whole way
    Linear,

    // starts fast and slows down as it arrives
    #[default]
    Out,

    // starts slow, speeds up in the middle and slows down again
    InOut,

    /*
     * a cubic bezier through (0, 0) and (1, 1) with these two handles,
     * like css's cubic-bezier(); Curve(0.2, 0.0, 0.0, 1.0) leaves quickly
     * and settles very softly
     */
    Curve(f32, f32, f32, f32),
}

impl Easing {
    // turns how much time has passed into how far along the way it is, both from 0 to 1
    pub(crate) fn apply(self, progress: f32) -> f32 {
        match self {
            Easing::Linear => progress,
            Easing::Out => Self::out(progress),
            Easing::InOut => Self::in_out(progress),
            Easing::Curve(x1, y1, x2, y2) => Self::curve(progress, [x1, y1, x2, y2]),
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

    /*
     * the curve is drawn by a parameter t, not by time, so first find the t
     * whose x is the time that has passed, then its y is how far along it is
     */
    fn curve(progress: f32, [x1, y1, x2, y2]: [f32; 4]) -> f32 {
        if progress <= 0.0 || progress >= 1.0 {
            return progress.clamp(0.0, 1.0);
        }

        // x only grows along the curve, so halving the range always closes in on t
        let mut low = 0.0;
        let mut high = 1.0;

        for _ in 0..24 {
            let middle = (low + high) / 2.0;

            if bezier(middle, x1, x2) < progress {
                low = middle;
            } else {
                high = middle;
            }
        }

        let t = (low + high) / 2.0;

        bezier(t, y1, y2)
    }
}

// one coordinate of a cubic bezier from 0 to 1, with its two handles at first and second
fn bezier(t: f32, first: f32, second: f32) -> f32 {
    let rest = 1.0 - t;

    3.0 * rest * rest * t * first + 3.0 * rest * t * t * second + t * t * t
}
