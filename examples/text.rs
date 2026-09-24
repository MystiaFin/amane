use amane::{App, Color, Column, LayerWindow, Rectangle, Text, children};

fn main() {
    App::new(LayerWindow::new(
        500,
        200,
        Column::new(children![
            Text {
                content: "Hello, Amane".into(),
                size: 48.0,
                color: Color::RED,
            },
            Rectangle {
                width: 200.0,
                height: 4.0,
                color: Color::GREEN,
            },
            Text {
                content: "gyp jumps below the line".into(),
                size: 24.0,
                color: Color::BLUE,
            },
        ]),
    ))
    .run();
}
