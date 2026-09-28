use std::sync::LazyLock;

use zbus::blocking::Connection;

/*
 * one connection per bus for the whole program,
 * every service thread shares it
 */
static SESSION: LazyLock<Connection> =
    LazyLock::new(|| Connection::session().expect("failed to connect to the session bus"));

static SYSTEM: LazyLock<Connection> =
    LazyLock::new(|| Connection::system().expect("failed to connect to the system bus"));

#[derive(Clone, Copy)]
pub struct Bus {
    pub(crate) connection: &'static Connection,
}

impl Bus {
    // the user's own programs: media players, notifications, the tray
    pub fn session() -> Self {
        Self {
            connection: &SESSION,
        }
    }

    // programs shared by every user: NetworkManager, UPower, logind
    pub fn system() -> Self {
        Self {
            connection: &SYSTEM,
        }
    }
}
