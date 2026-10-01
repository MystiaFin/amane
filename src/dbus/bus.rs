use std::sync::LazyLock;

use zbus::blocking::Connection;

/*
 * one connection per bus for the whole program, every service thread shares
 * it; none when the bus could not be reached, and then every call answers
 * with nothing. it is tried once, so a bus that starts later needs a restart
 */
static SESSION: LazyLock<Option<Connection>> = LazyLock::new(|| Connection::session().ok());

static SYSTEM: LazyLock<Option<Connection>> = LazyLock::new(|| Connection::system().ok());

#[derive(Clone, Copy)]
pub struct Bus {
    pub(crate) connection: Option<&'static Connection>,
}

impl Bus {
    // the user's own programs: media players, notifications, the tray
    pub fn session() -> Self {
        Self {
            connection: SESSION.as_ref(),
        }
    }

    // programs shared by every user: NetworkManager, UPower, logind
    pub fn system() -> Self {
        Self {
            connection: SYSTEM.as_ref(),
        }
    }
}
