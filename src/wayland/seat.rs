use smithay_client_toolkit::seat::{
    Capability, SeatHandler, SeatState, keyboard::KeyboardData, pointer::ThemeSpec,
};
use wayland_client::{Connection, Proxy, QueueHandle, protocol::wl_seat::WlSeat};

use crate::input::focus;

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
                // the fallback theme draws its icons on a surface of their own
                let cursor_surface = self.compositor.create_surface(qh);

                let pointer = self
                    .seat
                    .get_pointer_with_theme::<_, ()>(
                        qh,
                        &seat,
                        self.shm.wl_shm(),
                        cursor_surface,
                        ThemeSpec::default(),
                    )
                    .expect("failed to get pointer");

                self.pointer_device = Some(pointer);

                // files are dropped where the pointer is, so it comes with the pointer
                if let Some(manager) = &self.data_device_manager {
                    self.data_device = Some(manager.get_data_device(qh, &seat));
                }
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
                    pointer.pointer().release();
                }

                self.data_device = None;
            }

            Capability::Keyboard => {
                self.keyboard_repeat.cancel(&self.loop_handle);
                self.keyboard_focus = None;
                focus::clear();

                if let Some(keyboard) = self.keyboard_device.take() {
                    keyboard.release();
                }
            }

            _ => {}
        }
    }

    fn remove_seat(&mut self, connection: &Connection, qh: &QueueHandle<Self>, seat: WlSeat) {
        let owns_keyboard = self
            .keyboard_device
            .as_ref()
            .and_then(|keyboard| keyboard.data::<KeyboardData<Self, ()>>())
            .is_some_and(|keyboard| keyboard.seat() == &seat);

        if owns_keyboard {
            self.remove_capability(connection, qh, seat, Capability::Keyboard);
        }
    }
}
