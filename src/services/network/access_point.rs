use crate::Value;

use super::ssid;

// networkmanager's flag for a network that asks for a password
const PRIVACY: u32 = 1;

// a wifi network the device can see
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessPoint {
    pub(crate) ssid: String,

    // 0 to 100
    pub(crate) strength: u8,

    pub(crate) secured: bool,

    // the network the device is joined to right now
    pub(crate) active: bool,

    // networkmanager has a profile for it, password included
    pub(crate) saved: bool,
}

impl AccessPoint {
    pub(crate) fn from_properties(properties: &Value, active: bool) -> Self {
        let flags = properties.get("Flags").number() as u32;
        let wpa = properties.get("WpaFlags").number();
        let rsn = properties.get("RsnFlags").number();

        // the privacy flag alone means old wep, wpa and rsn cover everything newer
        let secured = flags & PRIVACY != 0 || wpa != 0.0 || rsn != 0.0;

        Self {
            ssid: ssid(properties.get("Ssid")),
            strength: properties.get("Strength").number() as u8,
            secured,
            active,
            saved: false,
        }
    }

    pub fn ssid(&self) -> &str {
        &self.ssid
    }

    pub fn strength(&self) -> u8 {
        self.strength
    }

    pub fn secured(&self) -> bool {
        self.secured
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn saved(&self) -> bool {
        self.saved
    }
}
