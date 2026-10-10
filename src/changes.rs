use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock, PoisonError};

use crate::timing;

// the backend sets this once, so services never name the event loop's types
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/*
 * which services changed since the windows last looked, and whether
 * something changed that no service names, which every window may show
 */
static CHANGED: Mutex<Changes> = Mutex::new(Changes {
    services: None,
    everything: false,
});

struct Changes {
    services: Option<HashSet<TypeId>>,
    everything: bool,
}

// what the window being drawn right now has read
thread_local! {
    static READ: RefCell<HashSet<TypeId>> = RefCell::new(HashSet::new());
}

pub fn set_waker(wake: impl Fn() + Send + Sync + 'static) {
    if WAKE.set(Box::new(wake)).is_err() {
        panic!("failed to set wake: already set");
    }

    // A service may have changed while the windows and GPU were being initialized.
    ping();
}

// asks the event loop to draw every window again, from any thread
pub fn mark_all() {
    if timing::enabled() {
        eprintln!("change everything");
    }

    CHANGED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .everything = true;

    ping();
}

// only the windows that read this service are drawn again
pub fn mark(service: TypeId) {
    let mut changes = CHANGED.lock().unwrap_or_else(PoisonError::into_inner);

    changes
        .services
        .get_or_insert_with(HashSet::new)
        .insert(service);

    drop(changes);

    ping();
}

// none means every window should draw
pub fn take() -> Option<HashSet<TypeId>> {
    let mut changes = CHANGED.lock().unwrap_or_else(PoisonError::into_inner);

    let services = changes.services.take().unwrap_or_default();

    if std::mem::take(&mut changes.everything) {
        return None;
    }

    Some(services)
}

pub fn note_read(service: TypeId) {
    READ.with_borrow_mut(|read| read.insert(service));
}

// what was read since the last call, which starts a fresh list
pub fn take_read() -> HashSet<TypeId> {
    READ.take()
}

fn ping() {
    // a write before the loop exists is picked up by the first frame anyway
    let Some(wake) = WAKE.get() else {
        return;
    };

    wake();
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    #[test]
    fn installing_the_waker_delivers_changes_made_during_startup() {
        struct Startup;
        mark(TypeId::of::<Startup>());
        let called = Arc::new(AtomicBool::new(false));
        let callback = called.clone();
        set_waker(move || callback.store(true, Ordering::SeqCst));
        assert!(called.load(Ordering::SeqCst));
    }
}
