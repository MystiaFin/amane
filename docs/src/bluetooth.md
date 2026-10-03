# Bluetooth

`Bluetooth` shows the adapter and its devices, and can power it, scan, pair, connect, and forget devices. It talks to BlueZ over D-Bus, so the `bluetooth` service has to be running.

It polls every 2 seconds.

## A device list

```rust
use amane::{App, Bluetooth, BluetoothDevice, Column, LayerWindow, Parent, Rectangle, Service, Text, Widget};

fn main() {
    Bluetooth::start_scan();

    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let bluetooth = Bluetooth::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for device in bluetooth.devices() {
        rows.push(Box::new(device_row(device)));
    }

    LayerWindow::new().width(360.0).height(400.0).child(Column::new(rows))
}

fn device_row(device: &BluetoothDevice) -> Rectangle {
    let state = if device.connected() {
        "connected"
    } else if device.paired() {
        "paired"
    } else {
        "new"
    };

    let path = String::from(device.path());
    let connected = device.connected();
    let paired = device.paired();

    Rectangle::new()
        .width(Parent)
        .height(30.0)
        .on_click(move |_| {
            if connected {
                Bluetooth::disconnect(&path);
            } else if paired {
                Bluetooth::connect(&path);
            } else {
                Bluetooth::pair(&path);
            }
        })
        .child(Text::new(format!("{}  {state}", device.name())))
}
```

## Reading Bluetooth state

| Function | Gives |
|---|---|
| `available()` | `false` when there's no adapter, or BlueZ isn't running |
| `powered()` | `true` when the adapter is on |
| `scanning()` | `true` while looking for new devices |
| `devices()` | known and discovered devices: connected first, then paired, then by name |

Each `BluetoothDevice` has:

| Function | Gives |
|---|---|
| `name()` | its name, like "WH-1000XM4" |
| `address()` | its hardware address |
| `path()` | its BlueZ path, which the control functions take |
| `icon()` | BlueZ's icon name for its type, like `"audio-headphones"` |
| `paired()` | `true` once paired |
| `connected()` | `true` while connected |
| `battery()` | its battery, 0 to 100, for devices that report one, otherwise `None` |

## Controlling devices

| Function | Does |
|---|---|
| `Bluetooth::set_powered(on)` | turns the adapter on or off |
| `Bluetooth::start_scan()` | starts looking for new devices. They show up in `devices()` as they're found. |
| `Bluetooth::stop_scan()` | stops looking |
| `Bluetooth::pair(path)` | pairs with a device, then connects, like other Bluetooth menus do |
| `Bluetooth::connect(path)` | connects a paired device |
| `Bluetooth::disconnect(path)` | disconnects a device |
| `Bluetooth::forget(path)` | unpairs and removes a device. It comes back only after a new scan. |

All of them run on a background thread and return right away. The result shows up in the next poll.

`pair` only works for devices that pair without a code, like headphones, mice, and most speakers. Devices that ask you to confirm or type a code, like phones and some keyboards, can't be paired from Amane yet. Pair those once with `bluetoothctl`, and then `connect` works.
