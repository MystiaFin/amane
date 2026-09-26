pub mod store;
mod ticker;

use std::sync::RwLockReadGuard;
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
}
