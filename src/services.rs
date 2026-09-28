pub mod store;
pub mod wake;
mod media;
mod network;
mod workspace;
mod workspaces;
mod write;

use std::sync::RwLockReadGuard;
use std::thread;
use std::time::Duration;

pub use media::Media;
pub use network::{Link, Network};
pub use workspace::Workspace;
pub use workspaces::Workspaces;
pub use write::Write;

pub trait Service: Send + Sync + Sized + 'static {
    fn new() -> Self;

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) {}

    /*
     * runs on the service's own thread, so waiting here never
     * freezes drawing; event-driven services replace it with their
     * own loop, and services that only change through input with
     * an empty one
     */
    fn listen() {
        loop {
            thread::sleep(Self::interval());

            Self::write().update();
        }
    }

    fn read() -> RwLockReadGuard<'static, Self> {
        store::find::<Self>()
            .read()
            .expect("failed to lock service")
    }

    /*
     * for input handlers: inside view() a read of the same
     * service is still held, so this would wait forever
     */
    fn write() -> Write<Self> {
        let guard = store::find::<Self>()
            .write()
            .expect("failed to lock service");

        Write { guard }
    }
}
