use smithay_client_toolkit::{
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};

use super::WaylandState;

impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    registry_handlers![OutputState];
}
