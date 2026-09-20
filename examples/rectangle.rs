use amane::{App, Color, LayerWindow, Rectangle};

fn main() {
    App::new(LayerWindow {
        width: 300,
        height: 120,

        child: Rectangle {
            width: 200.0,
            height: 60.0,
            color: Color::RED,
        },
    })
    .run();
}
