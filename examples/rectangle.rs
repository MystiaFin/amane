use amane::{App, Color, LayerWindow, Rectangle, Column, children};

fn main() {
    App::new(LayerWindow::new(
        500,
        200,
        Column::new(children![
            Rectangle {
                width: 100.0,
                height: 50.0,
                color: Color::RED,
            },
            Rectangle {
                width: 200.0,
                height: 30.0,
                color: Color::GREEN,
            },
            Rectangle {
                width: 150.0,
                height: 70.0,
                color: Color::BLUE,
            },
        ]),
    ))
    .run();
}
