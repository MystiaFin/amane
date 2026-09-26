use amane::{App, Color, Column, LayerWindow, Rectangle, Text, children};

fn main() {
    App::new().font("monospace").window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(500.0)
        .height(200.0)
        .child(Column::new(children![
            Text::new("Hello, Amane").size(48.0).color(Color::RED),
            Rectangle::new()
                .width(200.0)
                .height(4.0)
                .color(Color::GREEN),
            Text::new("this one overrides the default")
                .size(24.0)
                .color(Color::BLUE)
                .font("serif"),
        ]))
}
