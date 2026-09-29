mod blend;
mod easing;
pub mod moving;
mod spring;

use std::time::{Duration, Instant};

pub use blend::Blend;
pub use easing::Easing;
pub use spring::Spring;

pub struct Animation<T: Blend = f32> {
    pub(crate) from: T,
    pub(crate) target: T,

    pub(crate) started: Instant,
    pub(crate) duration: Duration,
    pub(crate) easing: Easing,
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
