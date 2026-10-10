use smithay_client_toolkit::reexports::calloop::{LoopHandle, ping::make_ping};

use crate::{app, changes};

use super::WaylandState;

pub fn insert(handle: &LoopHandle<'static, WaylandState>) {
    let (ping, source) = make_ping().expect("failed to create wake ping");

    handle
        .insert_source(source, |_, _, state| {
            if app::quit_requested() {
                state.running = false;
                return;
            }

            state.end_lock_if_unlocked();

            state.start_lock_if_asked();

            state.open_requested();

            state.close_requested();

            state.request_changed_frames();
        })
        .expect("failed to insert wake ping");

    changes::set_waker(move || ping.ping());
}
