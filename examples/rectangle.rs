use amane::{App, Color, Column, LayerWindow, Rectangle};

fn main() {
    let mut column = Column::new();

    column.push(Rectangle {
        width: 100.0,
        height: 50.0,
        color: Color::RED,
    });

    column.push(Rectangle {
        width: 200.0,
        height: 30.0,
        color: Color::GREEN,
    });

    column.push(Rectangle {
        width: 150.0,
        height: 70.0,
        color: Color::BLUE,
    });

    App::new(LayerWindow {
        width: 300,
        height: 200,
        child: column,
    })
    .run();
}
