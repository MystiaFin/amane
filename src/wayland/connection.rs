use wayland_client::Connection;

pub fn connect() -> Connection {
    Connection::connect_to_env().expect("failed to connect to Wayland")
}
