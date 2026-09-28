use amane::{
    App, Bluetooth, BluetoothDevice, Color, Column, Layer, LayerWindow, Parent, Rectangle, Service,
    Text, Widget,
};

// click the top line to turn bluetooth on or off, click a device to connect or disconnect it
fn main() {
    Bluetooth::start_scan();

    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let bluetooth = Bluetooth::read();

    let status = if !bluetooth.available() {
        "no bluetooth adapter"
    } else if bluetooth.powered() {
        "bluetooth on"
    } else {
        "bluetooth off"
    };

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    let powered = bluetooth.powered();

    rows.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(30.0)
            .fill(Color::BLUE)
            .on_click(move |_| Bluetooth::set_powered(!powered))
            .child(Text::new(status).size(16.0).color(Color::WHITE)),
    ));

    for device in bluetooth.devices() {
        rows.push(Box::new(device_row(device)));
    }

    LayerWindow::new()
        .width(360.0)
        .height(400.0)
        .layer(Layer::Top)
        .child(Column::new(rows))
}

fn device_row(device: &BluetoothDevice) -> Rectangle {
    let state = if device.connected() {
        "connected"
    } else if device.paired() {
        "paired"
    } else {
        "new"
    };

    let label = match device.battery() {
        Some(battery) => format!("{}  {state}  {battery}%", device.name()),
        None => format!("{}  {state}", device.name()),
    };

    let path = String::from(device.path());
    let connected = device.connected();
    let paired = device.paired();

    Rectangle::new()
        .width(Parent)
        .height(30.0)
        .fill(Color::BLACK)
        .on_click(move |_| {
            if connected {
                Bluetooth::disconnect(&path);
            } else if paired {
                Bluetooth::connect(&path);
            } else {
                Bluetooth::pair(&path);
            }
        })
        .child(Text::new(&label).size(14.0).color(Color::WHITE))
}
