use smithay_client_toolkit::reexports::protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
    ext_data_control_manager_v1::{self, ExtDataControlManagerV1},
    ext_data_control_offer_v1::{self, ExtDataControlOfferV1},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle, event_created_child, protocol::wl_seat::WlSeat,
};

use super::WaylandState;

// the manager never sends events, but wayland-client still needs somewhere to send them
impl Dispatch<ExtDataControlManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ExtDataControlManagerV1,
        _: ext_data_control_manager_v1::Event,
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
        _: ext_data_control_device_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
    event_created_child!(WaylandState, ExtDataControlDeviceV1, [
        ext_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, ()),
    ]);
}

impl Dispatch<ExtDataControlOfferV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ExtDataControlOfferV1,
        _: ext_data_control_offer_v1::Event,
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
