use amane::{App, Color, Column, LayerWindow, Rectangle, Text, children};

fn main() {
    App::new()
        .font("monospace")
        .window(LayerWindow::new(
            500,
            200,
            Column::new(children![
                Text {
                    content: "Hello, Amane".into(),
                    size: 48.0,
                    color: Color::RED,
                    font: None,
                },
                Rectangle {
                    width: 200.0,
                    height: 4.0,
                    color: Color::GREEN,
                },
                Text {
                    content: "this one overrides the default".into(),
                    size: 24.0,
                    color: Color::BLUE,
                    font: Some("serif".into()),
                },
            ]),
        ))
        .run();
}
