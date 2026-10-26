use amane::{
    App, Clipboard, Color, Column, Horizontal, Layer, LayerWindow, Parent, Rectangle, Service,
    Text, Vertical, Widget,
};

// copy some text in any app, and it shows up at the top; click a row to paste it
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let clipboard = Clipboard::read();

    let mut rows = Vec::new();

    for text in clipboard.list() {
        rows.push(row(text));
    }

    LayerWindow::new()
        .width(400.0)
        .height(600.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Top)
        .child(Column::new(rows).gap(8.0))
}

// clicking a row types it into the app that has focus
fn row(text: &str) -> Box<dyn Widget> {
    let entry = String::from(text);

    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(40.0)
            .fill(Color::BLUE)
            .on_click(move |_| Clipboard::paste(&entry))
            .child(Text::new(text).size(16.0).color(Color::WHITE).elide()),
    )
}
