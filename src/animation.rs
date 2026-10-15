use std::{
    f32::consts::PI,
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

use crate::Color;

pub struct Animation<T: Blend = f32> {
    pub(crate) from: T,
    pub(crate) target: T,

    pub(crate) started: Instant,
    pub(crate) duration: Duration,
    pub(crate) curve: Curve,
    spring_state: Mutex<Option<SpringState>>,
}

pub enum Curve {
    Bezier(Easing),
    Spring(Spring),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    stiffness: f32,
    damping: f32,
    mass: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpringState {
    progress: f32,
    velocity: f32,
    prev_frame: Instant,
}

impl<T: Blend> Animation<T> {
    // starts at rest on `value`, nothing moves until `to` is called
    pub fn new(value: T) -> Self {
        Self {
            from: value,
            target: value,
            started: Instant::now(),
            duration: Duration::from_millis(200),
            curve: Curve::Bezier(Easing::default()),
            spring_state: Mutex::new(None),
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;

        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.curve = Curve::Bezier(easing);

        self
    }

    pub fn spring(mut self, spring: Spring) -> Self {
        self.curve = Curve::Spring(spring);

        self
    }

    pub fn spring_bounciness(mut self, bounciness: f32) -> Self {
        assert!(
            bounciness.abs() < 1.0,
            "Spring bounciness must be between -1.0 and 1.0 non inclusive"
        );
        assert!(
            !self.duration.is_zero(),
            "Spring bounciness requires duration to be bigger than zero"
        );
        let pi = PI;
        let duration = self.duration.as_secs_f32();
        let stiffness = (2.0 * pi / duration).powf(2.0);
        let damping = if bounciness >= 0.0 {
            ((1.0 - bounciness) * 4.0 * pi) / duration
        } else {
            4.0 * pi / (duration * (1.0 + bounciness))
        };
        let mass = 1.0;
        let spring = Spring::verify_spring(stiffness, damping, mass);
        self.curve = Curve::Spring(spring);

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

        if matches!(self.curve, Curve::Spring(_)) {
            let mut spring_state = self
                .spring_state
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match spring_state.as_mut() {
                Some(state) => state.reset(),
                None => *spring_state = Some(SpringState::new()),
            }
        }
    }

    // read in the view, and while it is still moving the window keeps drawing frames
    pub fn value(&self) -> T {
        match self.curve {
            Curve::Bezier(_) => {
                if self.progress() < 1.0 {
                    moving::set();
                }
            }
            Curve::Spring(_) => {
                if self
                    .spring_state
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .is_some()
                {
                    moving::set();
                }
            }
        }
        self.current()
    }

    fn current(&self) -> T {
        match self.curve {
            Curve::Bezier(easing) => {
                T::blend(self.from, self.target, easing.apply(self.progress()))
            }
            Curve::Spring(spring) => {
                let mut spring_state = self
                    .spring_state
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let Some(state) = spring_state.as_mut() else {
                    return self.target;
                };
                if state.is_settled() {
                    *spring_state = None;
                    return self.target;
                }
                let amount = state.advance(&spring);
                T::blend(self.from, self.target, amount)
            }
        }
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
    fn apply(self, progress: f32) -> f32 {
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

impl SpringState {
    fn new() -> Self {
        let now = Instant::now();

        Self {
            progress: 0.0,
            velocity: 0.0,
            prev_frame: now,
        }
    }

    fn reset(&mut self) {
        self.progress = 0.0;
        self.velocity = 0.0;
        self.prev_frame = Instant::now();
    }

    fn advance(&mut self, spring: &Spring) -> f32 {
        let now = Instant::now();
        let time_delta = now
            .duration_since(self.prev_frame)
            .as_secs_f32()
            .clamp(0.0, 0.1);
        self.prev_frame = now;
        let displacement = self.progress - 1.0;
        let step = spring.apply(displacement, &mut self.velocity, time_delta);
        self.progress += step;
        if self.is_settled() {
            self.progress = 1.0;
            self.velocity = 0.0;
        }

        self.progress
    }

    fn is_settled(&self) -> bool {
        let displacement = self.progress - 1.0;
        displacement.abs() < 0.001 && self.velocity.abs() < 0.001
    }
}

impl Spring {
    pub fn new(stiffness: f32, damping_ratio: f32, mass: f32) -> Self {
        Self::verify_spring(stiffness, damping_ratio, mass)
    }

    fn apply(&self, displacement: f32, velocity: &mut f32, dt: f32) -> f32 {
        let spring_force = -displacement * self.stiffness;
        let damping_force = -*velocity * self.damping;
        let resultant_force = spring_force + damping_force;
        let acceleration = resultant_force / self.mass;

        *velocity += acceleration * dt;
        *velocity * dt
    }
    fn verify_spring(stiffness: f32, damping_ratio: f32, mass: f32) -> Self {
        assert!(
            damping_ratio > 0.0 && damping_ratio.is_finite() && damping_ratio <= 1.0,
            "Spring damping ratio must be positive and finite"
        );
        assert!(
            stiffness > 0.0 && stiffness.is_finite(),
            "Spring stiffness must be positive and finite"
        );
        assert!(
            mass > 0.0 && mass.is_finite(),
            "Spring mass must be positive and finite"
        );
        let damping = damping_ratio * 2.0 * (stiffness * mass).sqrt();
        Self {
            stiffness,
            damping,
            mass,
        }
    }
    // Spatial springs for animating movement (rotation, resizing, etc.)
    pub const FAST_SPATIAL: Self = Self {
        stiffness: 800.0,
        damping: 34.0,
        mass: 1.0,
    };

    pub const SLOW_SPATIAL: Self = Self {
        stiffness: 200.0,
        damping: 22.0,
        mass: 1.0,
    };
    pub const DEFAULT_SPATIAL: Self = Self {
        stiffness: 380.0,
        damping: 32.0,
        mass: 1.0,
    };

    // Effect springs for animating colors/opacity
    pub const FAST_EFFECTS: Self = Self {
        stiffness: 3800.0,
        damping: 123.0,
        mass: 1.0,
    };
    pub const DEFAULT_EFFECTS: Self = Self {
        stiffness: 1600.0,
        damping: 80.0,
        mass: 1.0,
    };
    pub const SLOW_EFFECTS: Self = Self {
        stiffness: 800.0,
        damping: 57.0,
        mass: 1.0,
    };
}

impl Default for Spring {
    fn default() -> Self {
        Self::DEFAULT_SPATIAL
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
