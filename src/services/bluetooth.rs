mod device;

use std::thread;
use std::time::Duration;

use crate::{Argument, Bus, Service, Value};

pub use device::BluetoothDevice;

const BLUEZ: &str = "org.bluez";
const ADAPTER: &str = "org.bluez.Adapter1";
const DEVICE: &str = "org.bluez.Device1";
const OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";

#[derive(Default)]
pub struct Bluetooth {
    // none when the machine has no bluetooth or bluez isn't running
    adapter: Option<String>,

    powered: bool,

    scanning: bool,

    devices: Vec<BluetoothDevice>,
}

/*
 * polled, one call lists the adapter and every device at once,
 * which is simpler than following each device's own signals
 */
impl Service for Bluetooth {
    fn new() -> Self {
        let mut bluetooth = Self::default();

        bluetooth.update();

        bluetooth
    }

    fn interval() -> Duration {
        Duration::from_secs(2)
    }

    fn update(&mut self) {
        let objects = Bus::system().call(BLUEZ, "/", OBJECT_MANAGER, "GetManagedObjects", &[]);

        let Value::Map(objects) = objects else {
            *self = Self::default();

            return;
        };

        self.adapter = None;
        self.devices.clear();

        for (path, interfaces) in &objects {
            let adapter = interfaces.get(ADAPTER);

            // the first adapter wins, most machines only have one
            if adapter != &Value::Nothing && self.adapter.is_none() {
                self.adapter = Some(path.clone());

                self.powered = adapter.get("Powered").bool();
                self.scanning = adapter.get("Discovering").bool();
            }

            if interfaces.get(DEVICE) != &Value::Nothing {
                self.devices.push(BluetoothDevice::from_interfaces(path, interfaces));
            }
        }

        // connected first, then paired, then by name
        self.devices.sort_by_key(|device| {
            let order = (!device.connected, !device.paired);

            (order, device.name.to_lowercase())
        });
    }
}

impl Bluetooth {
    pub fn available(&self) -> bool {
        self.adapter.is_some()
    }

    pub fn powered(&self) -> bool {
        self.powered
    }

    pub fn scanning(&self) -> bool {
        self.scanning
    }

    pub fn devices(&self) -> &[BluetoothDevice] {
        &self.devices
    }

    pub fn set_powered(powered: bool) {
        let Some(adapter) = adapter() else {
            return;
        };

        Bus::system().set_property(BLUEZ, &adapter, ADAPTER, "Powered", Argument::from(powered));

        Self::write().update();
    }

    // new devices show up in devices() while scanning
    pub fn start_scan() {
        adapter_call("StartDiscovery", &[]);
    }

    pub fn stop_scan() {
        adapter_call("StopDiscovery", &[]);
    }

    pub fn connect(device: &str) {
        device_call(device, "Connect");
    }

    pub fn disconnect(device: &str) {
        device_call(device, "Disconnect");
    }

    // pairs, then connects like other bluetooth menus do; only for devices that pair without a code
    pub fn pair(device: &str) {
        device_calls(device, &["Pair", "Connect"]);
    }

    // unpairs and removes the device, it comes back only after a new scan
    pub fn forget(device: &str) {
        let path = Argument::Path(String::from(device));

        adapter_call("RemoveDevice", &[path]);
    }
}

// looked up again on each call, so input handlers don't have to read the service
fn adapter() -> Option<String> {
    Bluetooth::read().adapter.clone()
}

fn adapter_call(method: &'static str, arguments: &[Argument]) {
    let Some(adapter) = adapter() else {
        return;
    };

    let arguments = arguments.to_vec();

    thread::spawn(move || {
        Bus::system().call(BLUEZ, &adapter, ADAPTER, method, &arguments);

        Bluetooth::write().update();
    });
}

// connecting can take seconds, so it runs on its own thread and the bar keeps drawing
fn device_call(device: &str, method: &'static str) {
    device_calls(device, &[method]);
}

// one after another, each waiting for the one before, like pairing before connecting
fn device_calls(device: &str, methods: &[&'static str]) {
    let device = String::from(device);

    let methods = methods.to_vec();

    thread::spawn(move || {
        for method in methods {
            Bus::system().call(BLUEZ, &device, DEVICE, method, &[]);

            Bluetooth::write().update();
        }
    });
}
