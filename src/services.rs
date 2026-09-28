pub mod store;
mod ticker;

use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

pub use ticker::Ticker;

pub trait Service: Send + Sync + Sized + 'static {
    fn new() -> Self;

    fn interval() -> Duration;

    fn update(&mut self);

    fn read() -> RwLockReadGuard<'static, Self> {
        store::find::<Self>()
            .read()
            .expect("failed to lock service")
    }

    /*
     * for input handlers: inside view() a read of the same
     * service is still held, so this would wait forever
     */
    fn write() -> RwLockWriteGuard<'static, Self> {
        store::find::<Self>()
            .write()
            .expect("failed to lock service")
    }
}
