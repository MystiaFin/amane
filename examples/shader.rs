use amane::{App, LayerWindow, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

// run from the project folder, the shader paths are relative to it
fn view() -> LayerWindow {
    LayerWindow::new()
        .width(420.0)
        .height(200.0)
        .child(Row::new(children![
            Rectangle::new()
                .width(200.0)
                .height(200.0)
                .radius(24.0)
                .shader("examples/shaders/waves.wgsl"),
            Rectangle::new()
                .width(200.0)
                .height(200.0)
                .radius(100.0)
                .rotate(15.0)
                .shader("examples/shaders/rings.frag")
        ]))
}
