use amane::{App, Color, LayerWindow, Parent, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(400.0)
        .height(400.0)
        .child(Row::new(children![
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .radius(20.0)
                .color(Color::RED)
        ]))
}
