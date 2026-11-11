use std::pin::pin;
use std::task::{Context, Poll, Waker};

// wgpu hands back answers that are already finished, so looking once is enough to get them
pub fn wait<T>(answer: impl Future<Output = T>) -> T {
    let mut context = Context::from_waker(Waker::noop());

    let Poll::Ready(value) = pin!(answer).poll(&mut context) else {
        panic!("failed to wait for the gpu");
    };

    value
}
