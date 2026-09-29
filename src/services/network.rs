mod access_point;
mod link;
mod manager;
mod profile;
mod wifi;

use crate::{Service, Value};

pub use access_point::AccessPoint;
pub use link::Link;

#[derive(Default)]
pub struct Network {
    link: Link,

    // empty on a wired link
    ssid: String,

    // 0 to 100, and 0 on a wired link
    strength: u8,

    wifi_enabled: bool,

    // joining a wifi network that isn't up yet
    connecting: bool,

    // empty without a wifi device, or with the radio off
    access_points: Vec<AccessPoint>,
}

/*
 * polled, because the access point's strength
 * changes all the time without a new connection
 */
impl Service for Network {
    fn new() -> Self {
        let mut network = Self::default();

        network.update();

        network
    }

    fn update(&mut self) {
        *self = Self::default();

        // networks to join are listed even while offline
        self.wifi_enabled = manager::wifi_enabled();

        if let Some(device) = wifi::device() {
            self.access_points = wifi::access_points(&device);
            self.connecting = wifi::connecting(&device);
        }

        if !manager::connected() {
            return;
        }

        let connection = manager::primary_connection();

        self.link = Link::from_type(connection.get("Type").text());

        if self.link != Link::Wifi {
            return;
        }

        let access_point = manager::access_point(connection.get("SpecificObject").text());

        self.ssid = ssid(access_point.get("Ssid"));
        self.strength = access_point.get("Strength").number() as u8;
    }
}

impl Network {
    pub fn connected(&self) -> bool {
        self.link != Link::Offline
    }

    pub fn link(&self) -> Link {
        self.link
    }

    pub fn ssid(&self) -> &str {
        &self.ssid
    }

    pub fn strength(&self) -> u8 {
        self.strength
    }

    pub fn wifi_enabled(&self) -> bool {
        self.wifi_enabled
    }

    pub fn connecting(&self) -> bool {
        self.connecting
    }

    pub fn access_points(&self) -> &[AccessPoint] {
        &self.access_points
    }
}

/*
 * these talk to networkmanager right away, and the
 * next update shows what changed
 */
impl Network {
    // new networks show up in access_points a few seconds later
    pub fn scan() {
        let Some(device) = wifi::device() else {
            return;
        };

        wifi::scan(&device);
    }

    /*
     * a saved profile already holds its password, so it is joined as it is;
     * otherwise networkmanager saves a new one, password included
     */
    pub fn connect(ssid: &str, password: Option<&str>) {
        let Some(device) = wifi::device() else {
            return;
        };

        if let Some(profile) = profile::find(ssid) {
            manager::activate(&profile, &device);

            return;
        }

        let settings = profile::settings(ssid, password);

        manager::add_and_activate(settings, &device);
    }

    pub fn disconnect() {
        let Some(device) = wifi::device() else {
            return;
        };

        wifi::disconnect(&device);
    }

    pub fn set_wifi(enabled: bool) {
        manager::set_wifi(enabled);
    }
}

// the ssid comes as raw bytes, and is almost always utf-8
fn ssid(bytes: &Value) -> String {
    let mut raw = Vec::new();

    for byte in bytes.list() {
        raw.push(byte.number() as u8);
    }

    String::from_utf8_lossy(&raw).into_owned()
}
