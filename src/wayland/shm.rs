use smithay_client_toolkit::shm::{Shm, ShmHandler};

use super::WaylandState;

// the cursor theme fallback draws its icons into shared memory
impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}
