use amane::{App, Color, Layer, LayerWindow, Rectangle, Row, Size, Text, Vertical, children};

fn main() {
    App::new()
        .window(
            LayerWindow::new(Size::Full, 30)
                .anchor_vertical(Vertical::Top)
                .layer(Layer::Top)
                .root(Row::new(children![
                    Rectangle {
                        width: 30.0,
                        height: 30.0,
                        color: Color::BLUE,
                    },
                    Text {
                        content: "Amane bar".into(),
                        size: 20.0,
                        color: Color::RED,
                        font: None,
                    },
                ])),
        )
        .run();
}
