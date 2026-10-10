use amane::{
    App, Center, Full, Layer, LayerWindow, Margin, Monitor, Parent, Rectangle, Text, Vertical,
    Window,
};

fn main() {
    let factor = std::env::var("AMANE_SCALE_FACTOR")
        .map(|value| value.parse::<f32>().expect("invalid AMANE_SCALE_FACTOR"))
        .unwrap_or(1.0);

    App::new()
        .scale_factor(factor)
        .window_per_monitor(bar)
        .normal_window("scale", panel)
        .run();
}

fn bar(monitor: &Monitor) -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            bottom: 8,
            left: 10,
            right: 10,
            ..Margin::default()
        })
        .layer(Layer::Top)
        .namespace("amane-scale")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#1e1e2e")
                .padding(5.0)
                .child(
                    Text::new(format!(
                        "{}: {} x {}",
                        monitor.name, monitor.width, monitor.height
                    ))
                    .size(16.0)
                    .color("#cdd6f4".into()),
                ),
        )
}

fn panel() -> Window {
    Window::new()
        .title("Amane scale example")
        .size(240.0, 120.0)
        .resizable(false)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#313244")
                .align_child(Center, Center)
                .child(Text::new("Scaled shell").size(20.0).color("#cdd6f4".into())),
        )
}
