use amane::{
    App, Color, Full, Horizontal, Layer, LayerWindow, Monitor, Parent, Rectangle, Text, Vertical,
};

fn main() {
    App::new().window_per_monitor(bar).window(corner).run();
}

// one bar on every monitor, showing which monitor it is on
fn bar(monitor: &Monitor) -> LayerWindow {
    let label = format!("{} {}x{}", monitor.name, monitor.width, monitor.height);

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .child(Text::new(label).size(20.0).color(Color::WHITE)),
        )
}

// a second window, on whichever monitor the compositor picks
fn corner() -> LayerWindow {
    LayerWindow::new()
        .width(200.0)
        .height(60.0)
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .child(Text::new("second window").size(18.0).color(Color::WHITE)),
        )
}
