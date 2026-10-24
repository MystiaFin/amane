use crate::{Clipboard, Service};

use std::io::{self, Read};
use std::os::fd::AsFd;
use std::thread;

use smithay_client_toolkit::reexports::protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{
        EVT_DATA_OFFER_OPCODE, Event as DeviceEvent, ExtDataControlDeviceV1,
    },
    ext_data_control_manager_v1::{Event as ManagerEvent, ExtDataControlManagerV1},
    ext_data_control_offer_v1::{Event as OfferEvent, ExtDataControlOfferV1},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle, event_created_child, protocol::wl_seat::WlSeat,
};

use super::WaylandState;

const MIME_TEXT_FORMAT: &str = "text/plain;charset=utf-8";

// the manager never sends events, but wayland-client still needs somewhere to send them
impl Dispatch<ExtDataControlManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ExtDataControlManagerV1,
        _: ManagerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtDataControlDeviceV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ExtDataControlDeviceV1,
        event: DeviceEvent,
        _: &(),
        connection: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let DeviceEvent::Selection { id } = event else {
            return;
        };

        // none when the clipboard was emptied
        let Some(offer) = id else {
            return;
        };

        let Ok((mut reader, writer)) = io::pipe() else {
            return;
        };

        offer.receive(String::from(MIME_TEXT_FORMAT), writer.as_fd());

        // the reader only finishes once every write end is closed, ours included
        drop(writer);

        // the receive request has to reach the other app before it can write anything
        let _ = connection.flush();

        offer.destroy();

        // a slow app, or amane itself once it copies, must not freeze the shell while it writes
        thread::spawn(move || {
            let mut text = String::new();

            let read = reader.read_to_string(&mut text);

            if read.is_err() {
                return;
            }
            Clipboard::write().add(text);
        });
    }
    event_created_child!(WaylandState, ExtDataControlDeviceV1, [
        EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, ()),
    ]);
}

impl Dispatch<ExtDataControlOfferV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ExtDataControlOfferV1,
        _: OfferEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl WaylandState {
    pub fn create_clipboard_device(&mut self, seat: &WlSeat) {
        let Some(manager) = &self.clipboard_manager else {
            return;
        };

        let device = manager.get_data_device(seat, &self.qh, ());

        self.clipboard_device = Some(device);
    }
}
