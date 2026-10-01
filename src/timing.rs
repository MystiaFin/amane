use std::env;
use std::sync::LazyLock;
use std::time::Instant;

// AMANE_FRAMES=1 prints a line for every drawn frame, to find where frame time goes
static ENABLED: LazyLock<bool> = LazyLock::new(|| env::var_os("AMANE_FRAMES").is_some());

pub fn enabled() -> bool {
    *ENABLED
}

// when a frame's steps finished, from the redraw starting to the gpu taking the frame
pub struct Timing {
    pub started: Instant,
    pub viewed: Instant,
    pub drawn: Instant,
    pub presented: Instant,
}

/*
 * gap is the time since this window's last frame, so a drop shows as a
 * gap near 33 ms or more; view, draw and gpu are this frame's own steps
 */
pub fn log(name: &str, width: u32, height: u32, last: Option<Instant>, timing: &Timing) {
    if !*ENABLED {
        return;
    }

    let milliseconds = |from: Instant, to: Instant| to.duration_since(from).as_secs_f32() * 1000.0;

    let gap = match last {
        Some(last) => milliseconds(last, timing.started),
        None => 0.0,
    };

    eprintln!(
        "frame {name} {width}x{height} gap {gap:.1} view {:.1} draw {:.1} gpu {:.1}",
        milliseconds(timing.started, timing.viewed),
        milliseconds(timing.viewed, timing.drawn),
        milliseconds(timing.drawn, timing.presented),
    );
}
