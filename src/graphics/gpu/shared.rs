use std::cell::{Cell, RefCell};
use std::rc::Rc;

use vello::wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions,
    Surface,
};
use vello::{AaSupport, RendererOptions};

use super::wait::wait;

/*
 * one gpu device and one vello for every window: separate devices are
 * separate gpu contexts, which the driver switches between whenever two
 * windows draw, and each compiled every shader again on its first frame
 */
#[derive(Clone)]
pub struct Shared {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub vello: Rc<RefCell<vello::Renderer>>,

    // how many times vello dropped its image atlas, which every window's images were in
    pub atlas_drops: Rc<Cell<u64>>,
}

// every window draws on the main thread, so thread locals are enough
thread_local! {
    static INSTANCE: Instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());

    static SHARED: RefCell<Option<Shared>> = const { RefCell::new(None) };
}

// the surfaces of every window come from the same instance
pub fn instance() -> Instance {
    INSTANCE.with(Instance::clone)
}

// opened by the first window, with a gpu that can draw to its surface; later windows reuse it
pub fn get(surface: &Surface) -> Shared {
    SHARED.with_borrow_mut(|shared| shared.get_or_insert_with(|| open(surface)).clone())
}

fn open(surface: &Surface) -> Shared {
    let adapter_options = RequestAdapterOptions {
        compatible_surface: Some(surface),
        ..RequestAdapterOptions::default()
    };

    let adapter = wait(instance().request_adapter(&adapter_options))
        .expect("failed to find a gpu that can draw to the window");

    // a software driver draws on the cpu and keeps every texture in ram, so say which one was picked
    let info = adapter.get_info();

    eprintln!("amane: drawing with {} ({:?}, {:?})", info.name, info.backend, info.device_type);

    let (device, queue) =
        wait(adapter.request_device(&DeviceDescriptor::default())).expect("failed to open gpu");

    // area anti-aliasing matches how edges looked with the cpu renderer
    let vello_options = RendererOptions {
        antialiasing_support: AaSupport::area_only(),
        ..RendererOptions::default()
    };

    let vello = vello::Renderer::new(&device, vello_options).expect("failed to start vello");

    Shared {
        adapter,
        device,
        queue,
        vello: Rc::new(RefCell::new(vello)),
        atlas_drops: Rc::new(Cell::new(0)),
    }
}
