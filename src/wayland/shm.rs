use std::{io::Write, os::fd::AsFd};

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_shm::{self, WlShm},
    },
};

use super::WaylandState;

pub fn create_buffer(
    shm: &WlShm,
    qh: &QueueHandle<WaylandState>,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> WlBuffer {
    let width = i32::try_from(width).expect("width too large");

    let height = i32::try_from(height).expect("height too large");

    let stride = width.checked_mul(4).expect("stride overflow");

    let size = stride.checked_mul(height).expect("buffer size overflow");

    assert_eq!(pixels.len(), size as usize, "pixel buffer has wrong size",);

    let mut file = tempfile::tempfile().expect("failed to create shm file");

    file.write_all(pixels).expect("failed to write pixels");

    file.flush().expect("failed to flush pixels");

    let pool = shm.create_pool(file.as_fd(), size, qh, ());

    let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, qh, ());

    pool.destroy();

    buffer
}

impl Dispatch<WlBuffer, ()> for WaylandState {
    fn event(
        _: &mut Self,
        buffer: &WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            buffer.destroy();
        }
    }
}
