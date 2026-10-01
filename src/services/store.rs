use std::any::{self, Any, TypeId};
use std::collections::HashMap;
use std::panic;
use std::sync::{LazyLock, Mutex, PoisonError, RwLock};
use std::thread;
use std::time::Duration;

use crate::Service;

static SERVICES: LazyLock<Mutex<HashMap<TypeId, &'static (dyn Any + Send + Sync)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn find<S: Service>() -> &'static RwLock<S> {
    let id = TypeId::of::<S>();

    if let Some(service) = SERVICES.lock().unwrap_or_else(PoisonError::into_inner).get(&id) {
        return service.downcast_ref().expect("failed to find service");
    }

    /*
     * services stay until the program exits,
     * so leaking gives a reference that is valid forever
     */
    let service = Box::leak(Box::new(RwLock::new(S::new())));

    SERVICES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(id, service);

    // stored first, so the thread's own reads and writes find this same service
    thread::spawn(keep_listening::<S>);

    service
}

// how long a service that panicked waits before it listens again
const RESTART_DELAY: Duration = Duration::from_secs(5);

/*
 * a listen that panics, like on a bus that went away, starts again after a
 * pause instead of leaving the service frozen; one that returns is done
 */
fn keep_listening<S: Service>() {
    while panic::catch_unwind(S::listen).is_err() {
        eprintln!("amane: {} stopped, starting it again", any::type_name::<S>());

        thread::sleep(RESTART_DELAY);
    }
}
