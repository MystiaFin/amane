mod graphics;
mod ui;
mod wayland;

use graphics::Color;
use ui::Rectangle;
use wayland::WaylandApp;

fn main() {
    let mut app = WaylandApp::new(Rectangle {
        width: 200.0,
        height: 60.0,
        color: Color::RED,
    });

    app.run();
}
