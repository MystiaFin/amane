mod wayland;
fn main() {
    let _connection = wayland::connection::connect();
}
