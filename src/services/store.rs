use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};
use std::thread;

use crate::Service;

static SERVICES: LazyLock<Mutex<HashMap<TypeId, &'static (dyn Any + Send + Sync)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn find<S: Service>() -> &'static RwLock<S> {
    let id = TypeId::of::<S>();

    if let Some(service) = SERVICES.lock().expect("failed to lock services").get(&id) {
        return service.downcast_ref().expect("failed to find service");
    }

    /*
     * services stay until the program exits,
     * so leaking gives a reference that is valid forever
     */
    let service = Box::leak(Box::new(RwLock::new(S::new())));

    SERVICES
        .lock()
        .expect("failed to lock services")
        .insert(id, service);

    // stored first, so the thread's own reads and writes find this same service
    thread::spawn(S::listen);

    service
}
