use amane::{App, Color, Rectangle};

fn main() {
    App::new(Rectangle {
        width: 200.0,
        height: 60.0,
        color: Color::RED,
    })
    .run();
}
