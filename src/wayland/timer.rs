use smithay_client_toolkit::reexports::calloop::{
    LoopHandle,
    timer::{TimeoutAction, Timer},
};

use crate::services::Ticker;

use super::WaylandState;

pub fn insert(handle: &LoopHandle<'static, WaylandState>, ticker: Ticker) {
    let timer = Timer::from_duration(ticker.interval);

    handle
        .insert_source(timer, move |_, _, state| {
            (ticker.update)();

            state.request_frame();

            TimeoutAction::ToDuration(ticker.interval)
        })
        .expect("failed to insert timer");
}
