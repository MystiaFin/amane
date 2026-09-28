use amane::{App, Gradient, LayerWindow, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(420.0)
        .height(200.0)
        .child(Row::new(children![
            // left to right, from blue to pink
            Rectangle::new()
                .width(200.0)
                .height(200.0)
                .radius(24.0)
                .fill(Gradient::linear(90.0, [(0.0, "#89b4fa"), (1.0, "#f5c2e7")])),
            Rectangle::new()
                .width(200.0)
                .height(200.0)
                .radius(100.0)
                .fill(Gradient::radial([
                    (0.0, "#f9e2af"),
                    (0.5, "#fab387"),
                    (1.0, "#1e1e2e"),
                ]))
        ]))
}
