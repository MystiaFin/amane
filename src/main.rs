mod wayland;

use wayland::WaylandApp;

fn main() {
    let app = WaylandApp::new();

    println!("Wayland initialized!");

    if app.has_surface() {
        println!("We have a surface!");
    }
}
