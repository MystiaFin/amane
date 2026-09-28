use smithay_client_toolkit::reexports::calloop::{LoopHandle, ping::make_ping};

use crate::services::wake;

use super::WaylandState;

pub fn insert(handle: &LoopHandle<'static, WaylandState>) {
    let (ping, source) = make_ping().expect("failed to create wake ping");

    handle
        .insert_source(source, |_, _, state| {
            state.request_frame();
        })
        .expect("failed to insert wake ping");

    wake::set(move || ping.ping());
}
