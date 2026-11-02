use crate::{Argument, Bus, Signal, Value};

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

/*
 * waits for each change a player announces, like play, pause or a new
 * track, and never ends unless the bus can't be reached
 */
pub fn changes() -> impl Iterator<Item = Signal> {
    let signals = Bus::session().signals("org.freedesktop.DBus.Properties", "PropertiesChanged");

    signals.filter(is_player_change)
}

// other programs announce their own properties too; the changed interface comes first
fn is_player_change(signal: &Signal) -> bool {
    let Some(interface) = signal.arguments().first() else {
        return false;
    };

    signal.path() == PATH && interface.text() == INTERFACE
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

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(path: &str, arguments: Vec<Value>) -> Signal {
        Signal {
            sender: String::from(":1.42"),
            path: String::from(path),
            arguments,
        }
    }

    #[test]
    fn reads_a_player_change() {
        let change = signal(
            PATH,
            vec![Value::Text(String::from(INTERFACE)), Value::Nothing],
        );

        assert!(is_player_change(&change));
    }

    #[test]
    fn skips_a_change_to_another_interface() {
        let change = signal(PATH, vec![Value::Text(String::from(ROOT_INTERFACE))]);

        assert!(!is_player_change(&change));
    }

    #[test]
    fn skips_a_change_on_another_path() {
        let change = signal(
            "/org/freedesktop/NetworkManager",
            vec![Value::Text(String::from(INTERFACE))],
        );

        assert!(!is_player_change(&change));
    }

    #[test]
    fn skips_a_change_without_arguments() {
        assert!(!is_player_change(&signal(PATH, Vec::new())));
        assert!(!is_player_change(&signal(PATH, vec![Value::Number(3.0)])));
    }
}
