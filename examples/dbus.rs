use amane::{
    App, Bus, Color, Full, Layer, LayerWindow, Parent, Rectangle, Service, Text, Vertical,
};

const NETWORK_MANAGER: &str = "/org/freedesktop/NetworkManager";

// try `nmcli radio wifi off` and `nmcli radio wifi on` while the bar is running
struct Wifi {
    enabled: bool,
}

impl Service for Wifi {
    fn new() -> Self {
        Self {
            enabled: read_enabled(),
        }
    }

    // NetworkManager announces each change, so nothing is polled
    fn listen() {
        let bus = Bus::system();

        for signal in bus.signals("org.freedesktop.DBus.Properties", "PropertiesChanged") {
            if signal.path() != NETWORK_MANAGER {
                continue;
            }

            let enabled = read_enabled();

            Self::write().enabled = enabled;
        }
    }
}

fn read_enabled() -> bool {
    let bus = Bus::system();

    let enabled = bus.property(
        "org.freedesktop.NetworkManager",
        NETWORK_MANAGER,
        "org.freedesktop.NetworkManager",
        "WirelessEnabled",
    );

    enabled.bool()
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let wifi = Wifi::read();

    let label = if wifi.enabled { "wifi on" } else { "wifi off" };

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
