use amane::{
    App, Button, Color, Column, Full, Layer, LayerWindow, Link, Network, Parent, Rectangle, Row,
    Service, Text, Vertical, Widget, children,
};

const BACKGROUND: Color = Color::rgb(0x1e, 0x1e, 0x2e);

const BUTTON: Color = Color::rgb(0x31, 0x32, 0x44);

const FOREGROUND: Color = Color::rgb(0xcd, 0xd6, 0xf4);

// put a password here to join a new secured network, saved ones already have theirs
const PASSWORD: Option<&str> = None;

// turn wifi off and on, or unplug the cable, while the bar is running
fn main() {
    App::new().window(bar).window(list).run();
}

fn bar() -> LayerWindow {
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

// click a network to join it
fn list() -> LayerWindow {
    let network = Network::read();

    let radio = if network.wifi_enabled() {
        "wifi off"
    } else {
        "wifi on"
    };

    let wifi_enabled = network.wifi_enabled();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    rows.push(Box::new(
        Row::new(children![
            button("scan", |_| Network::scan()),
            button(radio, move |_| Network::set_wifi(!wifi_enabled)),
            button("disconnect", |_| Network::disconnect()),
        ])
        .gap(8.0),
    ));

    for access_point in network.access_points() {
        let lock = if access_point.secured() {
            "locked"
        } else {
            "open"
        };

        let mark = if access_point.active() { "* " } else { "" };

        let label = format!(
            "{}{}  {}%  {}",
            mark,
            access_point.ssid(),
            access_point.strength(),
            lock
        );

        let ssid = String::from(access_point.ssid());

        rows.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(30.0)
                .fill(BUTTON)
                .padding(4.0)
                .on_click(move |_| Network::connect(&ssid, PASSWORD))
                .child(Text::new(label).size(18.0).color(FOREGROUND)),
        ));
    }

    LayerWindow::new().width(400.0).height(400.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(BACKGROUND)
            .padding(8.0)
            .child(Column::new(rows).width(Parent).gap(6.0)),
    )
}

fn button(label: &str, clicked: impl Fn(Button) + 'static) -> Rectangle {
    Rectangle::new()
        .width(110.0)
        .height(30.0)
        .fill(BUTTON)
        .padding(4.0)
        .on_click(clicked)
        .child(Text::new(label).size(18.0).color(FOREGROUND))
}
