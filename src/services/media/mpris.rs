use crate::{Argument, Bus, Value};

const PATH: &str = "/org/mpris/MediaPlayer2";

const INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

// where a player keeps its own name, like "Spotify"
const ROOT_INTERFACE: &str = "org.mpris.MediaPlayer2";

// every mpris player owns a bus name starting with this
const PREFIX: &str = "org.mpris.MediaPlayer2.";

// not a player, it mirrors the active one, so it would show that player twice
const PROXY: &str = "org.mpris.MediaPlayer2.playerctld";

// every player on the bus, sorted so they keep their order between polls
pub fn names() -> Vec<String> {
    let bus = Bus::session();

    let names = bus.call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListNames",
        &[],
    );

    let mut players = Vec::new();

    for name in names.list() {
        let name = name.text();

        if name.starts_with(PREFIX) && name != PROXY {
            players.push(String::from(name));
        }
    }

    players.sort();

    players
}

// all player properties in one call, as a map
pub fn properties(player: &str) -> Value {
    get_all(player, INTERFACE)
}

pub fn identity(player: &str) -> String {
    let root = get_all(player, ROOT_INTERFACE);

    String::from(root.get("Identity").text())
}

pub fn send(player: &str, method: &str) {
    Bus::session().call(player, PATH, INTERFACE, method, &[]);
}

fn get_all(player: &str, interface: &str) -> Value {
    let arguments = [Argument::from(interface)];

    Bus::session().call(
        player,
        PATH,
        "org.freedesktop.DBus.Properties",
        "GetAll",
        &arguments,
    )
}
