use crate::Value;

// a phone, headset or other device bluez knows about
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDevice {
    // the d-bus object, used to connect, pair and forget
    pub(crate) path: String,

    pub(crate) name: String,

    pub(crate) address: String,

    // a freedesktop icon name like "audio-headset"
    pub(crate) icon: String,

    pub(crate) paired: bool,

    pub(crate) connected: bool,

    // 0 to 100, only for devices that report it
    pub(crate) battery: Option<u8>,
}

impl BluetoothDevice {
    // interfaces is what bluez lists for one object: interface name -> properties
    pub(crate) fn from_interfaces(path: &str, interfaces: &Value) -> Self {
        let device = interfaces.get("org.bluez.Device1");
        let battery = interfaces.get("org.bluez.Battery1");

        let address = device.get("Address").text();

        // unnamed devices show their address, like other bluetooth menus do
        let name = match device.get("Alias").text() {
            "" => address,
            alias => alias,
        };

        let battery = match battery.get("Percentage") {
            Value::Nothing => None,
            percent => Some(percent.number() as u8),
        };

        Self {
            path: String::from(path),
            name: String::from(name),
            address: String::from(address),
            icon: String::from(device.get("Icon").text()),
            paired: device.get("Paired").bool(),
            connected: device.get("Connected").bool(),
            battery,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn icon(&self) -> &str {
        &self.icon
    }

    pub fn paired(&self) -> bool {
        self.paired
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    pub fn battery(&self) -> Option<u8> {
        self.battery
    }
}
