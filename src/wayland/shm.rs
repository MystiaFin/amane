use std::{io::Write, os::fd::AsFd};

use wayland_client::{
    QueueHandle,
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{self, WlShm},
    },
};

use super::WaylandState;

pub fn create_solid_buffer(
    shm: &WlShm,
    qh: &QueueHandle<WaylandState>,
    width: u32,
    height: u32,
) -> WlBuffer {
    let width_i32 = i32::try_from(width).expect("buffer width too large");

    let height_i32 = i32::try_from(height).expect("buffer height too large");

    let stride = width_i32.checked_mul(4).expect("buffer stride overflow");

    let size = stride
        .checked_mul(height_i32)
        .expect("buffer size overflow");

    /*
     * Temporary file backing our shared memory.
     */
    let mut file = tempfile::tempfile().expect("failed to create shm file");

    /*
     * ARGB8888:
     *
     * A = FF
     * R = FF
     * G = 00
     * B = 00
     *
     * → opaque red
     */
    let red = 0xFFFF0000u32.to_ne_bytes();

    for _ in 0..(u64::from(width) * u64::from(height)) {
        file.write_all(&red).expect("failed to write pixel data");
    }

    file.flush().expect("failed to flush pixel data");

    /*
     * Tell Wayland:
     *
     * "this file contains memory
     *  that we can share."
     */
    let pool = shm.create_pool(file.as_fd(), size, qh, ());

    /*
     * Define a rectangular wl_buffer
     * inside that memory.
     */
    let buffer = pool.create_buffer(
        0,
        width_i32,
        height_i32,
        stride,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );

    /*
     * The wl_buffer keeps the server-side
     * memory alive, so the pool object itself
     * is no longer needed.
     */
    pool.destroy();

    buffer
}
