use smithay_client_toolkit::shm::{
    Shm, ShmHandler,
    slot::{Buffer, SlotPool},
};
use wayland_client::protocol::wl_shm::Format;

use super::WaylandState;

pub fn create_buffer(pool: &mut SlotPool, width: u32, height: u32, pixels: &[u8]) -> Buffer {
    let width = i32::try_from(width).expect("width too large");

    let height = i32::try_from(height).expect("height too large");

    let stride = width.checked_mul(4).expect("stride overflow");

    let (buffer, canvas) = pool
        .create_buffer(width, height, stride, Format::Argb8888)
        .expect("failed to create buffer");

    canvas.copy_from_slice(pixels);

    buffer
}

impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}
