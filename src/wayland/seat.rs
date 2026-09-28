use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use wayland_client::{Connection, QueueHandle, protocol::wl_seat::WlSeat};

use super::WaylandState;

impl SeatHandler for WaylandState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer => {
                let pointer = self
                    .seat
                    .get_pointer(qh, &seat)
                    .expect("failed to get pointer");

                self.pointer_device = Some(pointer);
            }

            Capability::Keyboard => {
                let keyboard = self
                    .seat
                    .get_keyboard(qh, &seat, None)
                    .expect("failed to get keyboard");

                self.keyboard_device = Some(keyboard);
            }

            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer => {
                if let Some(pointer) = self.pointer_device.take() {
                    pointer.release();
                }
            }

            Capability::Keyboard => {
                if let Some(keyboard) = self.keyboard_device.take() {
                    keyboard.release();
                }
            }

            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}
