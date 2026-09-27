use amane::{App, Image, LayerWindow, Parent, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(600.0)
        .height(200.0)
        .child(Row::new(children![
            Rectangle::new()
                .width(200.0)
                .height(Parent)
                .radius(20.0)
                .fill(Image::cover("examples/wallpaper.png")),
            Rectangle::new()
                .width(200.0)
                .height(Parent)
                .fill(Image::contain("examples/wallpaper.png")),
            Rectangle::new()
                .width(200.0)
                .height(Parent)
                .radius(20.0)
                .border(4.0, "#ffffff")
                .fill(Image::stretch("examples/wallpaper.png")),
        ]))
}
