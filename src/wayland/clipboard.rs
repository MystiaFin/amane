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
use smithay_client_toolkit::reexports::protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::{Event as InputMethodManagerEvent, ZwpInputMethodManagerV2},
    zwp_input_method_v2::{Event as InputMethodEvent, ZwpInputMethodV2},
};
use wayland_client::protocol::wl_callback::{Event as CallbackEvent, WlCallback};
use wayland_client::{
    Connection, Dispatch, QueueHandle, event_created_child, protocol::wl_seat::WlSeat,
};

use super::WaylandState;

const MIME_TEXT_FORMAT: &str = "text/plain;charset=utf-8";

// commit_string refuses anything longer, a wayland message has to fit in 4096 bytes
const MAX_PASTE_BYTES: usize = 4000;

/*
 * a seat has one input method slot, and fcitx5 or ibus need it
 * the rest of the time, so it's only held while one paste is typed
 */
pub struct ClipboardPaste {
    input_method: ZwpInputMethodV2,
    text: String,

    // a text box in the focused app is ready to be typed into
    active: bool,

    // how many done events arrived, which commit has to send back
    done_count: u32,
}

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

    // runs on every wake, since that is when a Clipboard::paste shows up
    pub fn paste_if_asked(&mut self) {
        let Some(text) = Clipboard::take_paste_request() else {
            return;
        };

        // no input method on this compositor, so nothing can type it yet
        let Some(manager) = &self.input_method_manager else {
            return;
        };

        if text.len() > MAX_PASTE_BYTES {
            return;
        }

        // a paste still waiting for its answer holds the slot, a second one would only get unavailable
        if self.clipboard_paste.is_some() {
            return;
        }

        let Some(seat) = self.seat.seats().next() else {
            return;
        };

        let input_method = manager.get_input_method(&seat, &self.qh, ());

        // by the time this is answered, it's known whether a text box is ready
        self.connection.display().sync(&self.qh, ());

        self.clipboard_paste = Some(ClipboardPaste {
            input_method,
            text,
            active: false,
            done_count: 0,
        });
    }
}

// the manager never sends events, but wayland-client still needs somewhere to send them
impl Dispatch<ZwpInputMethodManagerV2, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpInputMethodManagerV2,
        _: InputMethodManagerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodV2,
        event: InputMethodEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(paste) = &mut state.clipboard_paste else {
            return;
        };

        /*
         * unavailable means fcitx5 or ibus has the slot; it needs nothing here,
         * active stays false, so the paste gives up
         */
        match event {
            InputMethodEvent::Activate => paste.active = true,
            InputMethodEvent::Deactivate => paste.active = false,
            InputMethodEvent::Done => paste.done_count += 1,
            _ => {}
        }
    }
}

// the only sync amane sends, answered after every event about the new input method
impl Dispatch<WlCallback, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &WlCallback,
        event: CallbackEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let CallbackEvent::Done { .. } = event else {
            return;
        };

        let Some(clipboard_paste) = state.clipboard_paste.take() else {
            return;
        };

        if clipboard_paste.active {
            clipboard_paste.input_method.commit_string(clipboard_paste.text);

            // the text only lands once commit confirms it, with how many done events were seen
            clipboard_paste.input_method.commit(clipboard_paste.done_count);
        }

        // gives the slot back, so fcitx5 or ibus can take it again
        clipboard_paste.input_method.destroy();
    }
}
