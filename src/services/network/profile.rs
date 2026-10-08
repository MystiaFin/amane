use std::collections::BTreeMap;

use crate::{Argument, Bus};

use super::manager::NAME;
use super::ssid;

const SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";

const SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";

const CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";

// a saved profile for this network, networkmanager keeps its password
pub fn find(name: &str) -> Option<String> {
    for (path, ssid) in wifi_profiles() {
        if ssid == name {
            return Some(path);
        }
    }

    None
}

// the names of every network with a saved profile
pub fn saved_names() -> Vec<String> {
    let mut names = Vec::new();

    for (_, ssid) in wifi_profiles() {
        names.push(ssid);
    }

    names
}

// each saved wifi profile's path with its network name
fn wifi_profiles() -> Vec<(String, String)> {
    let profiles = Bus::system().call(NAME, SETTINGS_PATH, SETTINGS, "ListConnections", &[]);

    let mut found = Vec::new();

    for profile in profiles.list() {
        let path = profile.text();

        let settings = Bus::system().call(NAME, path, CONNECTION, "GetSettings", &[]);

        // wired and vpn profiles have no wifi group, so their name is empty
        let wireless = settings.get("802-11-wireless");

        let name = ssid(wireless.get("ssid"));

        if name.is_empty() {
            continue;
        }

        found.push((String::from(path), name));
    }

    found
}

// only what networkmanager cannot work out itself, it fills in the rest
pub fn settings(name: &str, password: Option<&str>) -> Argument {
    let mut connection = BTreeMap::new();

    connection.insert(String::from("id"), Argument::from(name));
    connection.insert(String::from("type"), Argument::from("802-11-wireless"));

    let mut wireless = BTreeMap::new();

    wireless.insert(
        String::from("ssid"),
        Argument::Bytes(name.as_bytes().to_vec()),
    );

    let mut groups = BTreeMap::new();

    groups.insert(String::from("connection"), connection);
    groups.insert(String::from("802-11-wireless"), wireless);

    // an open network needs no security group at all
    if let Some(password) = password {
        let mut security = BTreeMap::new();

        security.insert(String::from("key-mgmt"), Argument::from("wpa-psk"));
        security.insert(String::from("psk"), Argument::from(password));

        groups.insert(String::from("802-11-wireless-security"), security);
    }

    Argument::Groups(groups)
}
