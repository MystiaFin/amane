use crate::{Argument, Bus, Value};

const PATH: &str = "/org/mpris/MediaPlayer2";

const INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

// every mpris player owns a bus name starting with this
const PREFIX: &str = "org.mpris.MediaPlayer2.";

// the first player on the bus, when several are open
pub fn find() -> Option<String> {
    let bus = Bus::session();

    let names = bus.call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListNames",
        &[],
    );

    for name in names.list() {
        if name.text().starts_with(PREFIX) {
            return Some(String::from(name.text()));
        }
    }

    None
}

// all player properties in one call, as a map
pub fn properties(player: &str) -> Value {
    let bus = Bus::session();

    let arguments = [Argument::from(INTERFACE)];

    bus.call(
        player,
        PATH,
        "org.freedesktop.DBus.Properties",
        "GetAll",
        &arguments,
    )
}

// with no player open there is nothing to control
pub fn send(method: &str) {
    let Some(player) = find() else {
        return;
    };

    Bus::session().call(&player, PATH, INTERFACE, method, &[]);
}
