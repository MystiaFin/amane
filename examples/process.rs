use amane::{
    App, Button, Color, Full, Layer, LayerWindow, Parent, Rectangle, Row, Service, Text, Vertical,
    children,
};

struct Niri {
    kernel: String,

    // the last thing niri reported, like a focus or workspace change
    event: String,
}

impl Service for Niri {
    fn new() -> Self {
        Self {
            kernel: amane::output("uname -r"),

            event: String::from("waiting for niri"),
        }
    }

    fn listen() {
        for line in amane::lines("niri msg event-stream") {
            Self::write().event = line;
        }
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let niri = Niri::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Row::new(children![
            Rectangle::new()
                .width(200.0)
                .height(Parent)
                .fill(Color::BLUE)
                .on_click(next_workspace)
                .child(Text::new(&niri.kernel).size(20.0).color(Color::WHITE)),
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .child(Text::new(&niri.event).size(20.0).color(Color::WHITE)),
        ]))
}

fn next_workspace(_: Button) {
    amane::spawn("niri msg action focus-workspace-down");
}
