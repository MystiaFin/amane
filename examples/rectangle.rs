use amane::{App, Color, LayerWindow, Parent, Rectangle, Row, children};

fn main() {
    App::new()
        .window(
            LayerWindow::new()
                .width(400.0)
                .height(400.0)
                .child(Row::new(children![
                    Rectangle::new()
                        .width(Parent)
                        .height(Parent)
                        .radius(20.0)
                        .border(2.0, Color::WHITE)
                        .color(Color::RED)
                        .opacity(0.5)
                ])),
        )
        .run();
}
