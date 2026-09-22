use amane::{App, Color, Direction, LayerWindow, Rectangle, children};

fn main() {
    App::new(
        LayerWindow::new(
            500,
            200,
            children![
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
            ],
        )
        .direction(Direction::Column),
    )
    .run();
}
