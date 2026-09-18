use wayland_client::Connection;

pub fn connect() -> connection {
    Connection::connect_to_env().expect("failed to connect to Wayland")
}
