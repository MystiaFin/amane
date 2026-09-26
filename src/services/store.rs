use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};

use crate::Service;
use crate::services::Ticker;

static SERVICES: LazyLock<Mutex<HashMap<TypeId, &'static (dyn Any + Send + Sync)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static STARTED: LazyLock<Mutex<Vec<Ticker>>> = LazyLock::new(|| Mutex::new(Vec::new()));

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

    let ticker = Ticker {
        interval: S::interval(),
        update: update::<S>,
    };

    STARTED
        .lock()
        .expect("failed to lock started services")
        .push(ticker);

    service
}

// hands the event loop every service created since it last asked
pub fn take_started() -> Vec<Ticker> {
    let mut started = STARTED.lock().expect("failed to lock started services");

    std::mem::take(&mut *started)
}

fn update<S: Service>() {
    let mut service = find::<S>().write().expect("failed to lock service");

    service.update();
}
