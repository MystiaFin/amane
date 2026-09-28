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
