use std::pin::pin;
use std::task::{Context, Poll, Waker};

// wgpu's native futures are already done when they are made, so one poll finishes them
pub(super) fn wait<T>(future: impl Future<Output = T>) -> T {
    let mut context = Context::from_waker(Waker::noop());

    let Poll::Ready(value) = pin!(future).poll(&mut context) else {
        panic!("failed to wait for the gpu");
    };

    value
}
