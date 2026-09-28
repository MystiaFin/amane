mod link;
mod manager;

use crate::{Service, Value};

pub use link::Link;

#[derive(Default)]
pub struct Network {
    link: Link,

    // empty on a wired link
    ssid: String,

    // 0 to 100, and 0 on a wired link
    strength: u8,
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
}

// the ssid comes as raw bytes, and is almost always utf-8
fn ssid(bytes: &Value) -> String {
    let mut raw = Vec::new();

    for byte in bytes.list() {
        raw.push(byte.number() as u8);
    }

    String::from_utf8_lossy(&raw).into_owned()
}
