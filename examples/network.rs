use amane::{
    App, Color, Full, Layer, LayerWindow, Link, Network, Parent, Rectangle, Service, Text,
    Vertical,
};

const BACKGROUND: Color = Color::rgb(0x1e, 0x1e, 0x2e);

const FOREGROUND: Color = Color::rgb(0xcd, 0xd6, 0xf4);

// turn wifi off and on, or unplug the cable, while the bar is running
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let network = Network::read();

    let label = match network.link() {
        Link::Offline => String::from("offline"),
        Link::Wired => String::from("wired"),
        Link::Wifi => format!("{}  {}%", network.ssid(), network.strength()),
        Link::Other => String::from("connected"),
    };

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(BACKGROUND)
                .child(Text::new(label).size(20.0).color(FOREGROUND)),
        )
}
