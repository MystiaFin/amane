mod wayland;

use wayland::WaylandApp;

fn main() {
    let mut app = WaylandApp::new();

    app.run();
}
