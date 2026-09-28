use crate::{Argument, Bus, Value};

const NAME: &str = "org.freedesktop.NetworkManager";

const PATH: &str = "/org/freedesktop/NetworkManager";

// networkmanager's own state number for "online, with internet"
const CONNECTED_GLOBAL: f64 = 70.0;

pub fn connected() -> bool {
    let state = Bus::system().property(NAME, PATH, NAME, "State");

    state.number() == CONNECTED_GLOBAL
}

// the connection that carries the default route, as a map of its properties
pub fn primary_connection() -> Value {
    let primary = Bus::system().property(NAME, PATH, NAME, "PrimaryConnection");

    properties(
        primary.text(),
        "org.freedesktop.NetworkManager.Connection.Active",
    )
}

// the wifi network a connection is joined to, as a map of its properties
pub fn access_point(path: &str) -> Value {
    properties(path, "org.freedesktop.NetworkManager.AccessPoint")
}

fn properties(path: &str, interface: &str) -> Value {
    let arguments = [Argument::from(interface)];

    Bus::system().call(
        NAME,
        path,
        "org.freedesktop.DBus.Properties",
        "GetAll",
        &arguments,
    )
}
