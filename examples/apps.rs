use amane::{
    Apps, App, Color, Column, Image, LayerWindow, Pointer, Rectangle, Row, ScrollArea, Service,
    Text, Widget, children,
};

fn main() {
    App::new().window(view).run();
}

// every program in the menu, click one to start it
fn view() -> LayerWindow {
    let apps = Apps::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for app in apps.list() {
        let launched = app.clone();

        let icon = Rectangle::new().width(24.0).height(24.0);

        let icon = match app.icon_path() {
            Some(path) => icon.fill(Image::contain(path)),
            None => icon,
        };

        let row = Rectangle::new()
            .width(300.0)
            .height(32.0)
            .fill("#1e1e2e")
            .cursor(Pointer)
            .on_click(move |_| launched.launch())
            .child(Row::new(children![
                icon,
                Text::new(app.name()).size(16.0).color(Color::WHITE),
            ]));

        rows.push(Box::new(row));
    }

    LayerWindow::new()
        .width(300.0)
        .height(500.0)
        .child(ScrollArea::new("apps", Column::new(rows)))
}
