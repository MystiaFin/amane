use amane::{App, Color, LayerWindow, Pointer, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(480.0)
        .height(160.0)
        .child(Row::new(children![
            // the pointer only turns into a hand over the tilted shape, not its old box
            Rectangle::new()
                .width(160.0)
                .height(160.0)
                .fill(Color::RED)
                .rotate(30.0)
                .scale(0.6)
                .cursor(Pointer)
                .on_click(|button| println!("clicked the rotated square with {button:?}")),
            Rectangle::new()
                .width(160.0)
                .height(160.0)
                .fill(Color::GREEN)
                .scale(0.5),
            Rectangle::new()
                .width(160.0)
                .height(160.0)
                .fill(Color::BLUE)
                .scale(0.5)
                .translate(20.0, -30.0)
        ]))
}
