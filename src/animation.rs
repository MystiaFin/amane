use std::time::{Duration, Instant};

use crate::Color;

pub struct Animation<T: Blend = f32> {
    pub(crate) from: T,
    pub(crate) target: T,

    pub(crate) started: Instant,
    pub(crate) duration: Duration,
    pub(crate) easing: Easing,
}

// anything an animation can move between, `amount` goes from 0 at `from` to 1 at `to`
pub trait Blend: Copy + PartialEq + Send + Sync + 'static {
    fn blend(from: Self, to: Self, amount: f32) -> Self;
}

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

impl<T: Blend> Animation<T> {
    // starts at rest on `value`, nothing moves until `to` is called
    pub fn new(value: T) -> Self {
        Self {
            from: value,
            target: value,

            started: Instant::now(),
            duration: Duration::from_millis(200),
            easing: Easing::default(),
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;

        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;

        self
    }

    // a change halfway through starts from where it is now, so it never jumps
    pub fn to(&mut self, target: T) {
        if target == self.target {
            return;
        }

        self.from = self.current();
        self.target = target;
        self.started = Instant::now();
    }

    // read in the view, and while it is still moving the window keeps drawing frames
    pub fn value(&self) -> T {
        if self.progress() < 1.0 {
            moving::set();
        }

        self.current()
    }

    fn current(&self) -> T {
        let eased = self.easing.apply(self.progress());

        T::blend(self.from, self.target, eased)
    }

    // 0 when it has just started, 1 once it has arrived
    fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }

        let elapsed = self.started.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();

        f32::min(elapsed / total, 1.0)
    }
}

impl Blend for f32 {
    fn blend(from: Self, to: Self, amount: f32) -> Self {
        from + (to - from) * amount
    }
}

impl Blend for Color {
    fn blend(from: Self, to: Self, amount: f32) -> Self {
        let r = blend_channel(from.r, to.r, amount);
        let g = blend_channel(from.g, to.g, amount);
        let b = blend_channel(from.b, to.b, amount);
        let a = blend_channel(from.a, to.a, amount);

        Color::rgba(r, g, b, a)
    }
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

/*
 * asks for one more frame after this one; call it from the view while
 * something you move yourself has not arrived, and stop calling it once
 * it has, so the window can rest
 */
pub fn request_frame() {
    moving::set();
}

fn blend_channel(from: u8, to: u8, amount: f32) -> u8 {
    let from = f32::from(from);
    let to = f32::from(to);

    let blended = f32::blend(from, to, amount);

    blended.round() as u8
}

pub mod moving {
    use std::sync::atomic::{AtomicBool, Ordering};

    // set by any animation the view read that has not arrived yet
    static MOVING: AtomicBool = AtomicBool::new(false);

    pub fn set() {
        MOVING.store(true, Ordering::Relaxed);
    }

    // asked once after each view, so the flag only ever describes that view
    pub fn take() -> bool {
        MOVING.swap(false, Ordering::Relaxed)
    }
}
