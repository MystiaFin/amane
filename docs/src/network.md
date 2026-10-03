# Network

`Network` shows the current connection and lists Wi-Fi networks, and it can join, leave, and scan. It talks to NetworkManager over D-Bus, so NetworkManager has to be running. Without it, everything reads as offline.

It polls once a second, because Wi-Fi strength changes all the time without anything announcing it. A poll that finds nothing new doesn't redraw anything.

## A connection label

```rust
use amane::{App, Full, LayerWindow, Link, Network, Service, Text};

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

    LayerWindow::new().width(Full).height(30.0).child(Text::new(label))
}
```

`Link::Other` covers connections that aren't wired or Wi-Fi, like a VPN or a phone's USB tethering.

## Reading the connection

| Function | Gives |
|---|---|
| `connected()` | `true` when there's any connection |
| `link()` | `Link::Offline`, `Wired`, `Wifi`, or `Other` |
| `ssid()` | the Wi-Fi network's name, empty on a wired link |
| `strength()` | Wi-Fi signal, 0 to 100, and 0 on a wired link |
| `wifi_enabled()` | `false` when the Wi-Fi radio is off |
| `connecting()` | `true` while joining a network |
| `access_points()` | the Wi-Fi networks in range, empty with no Wi-Fi device or with the radio off |

Each `AccessPoint` in `access_points()` has:

| Function | Gives |
|---|---|
| `ssid()` | the network's name |
| `strength()` | signal, 0 to 100 |
| `secured()` | `true` when it needs a password |
| `active()` | `true` for the network you're on |
| `saved()` | `true` when NetworkManager already has a profile for it |

## Joining and leaving networks

| Function | Does |
|---|---|
| `Network::scan()` | looks for networks. New ones show up in `access_points()` a few seconds later. |
| `Network::connect(ssid, password)` | joins a network |
| `Network::disconnect()` | leaves the current Wi-Fi network |
| `Network::set_wifi(enabled)` | turns the Wi-Fi radio on or off |

All of them run on a background thread and return right away. The result shows up in the next poll.

`connect` takes the password as an `Option`:

- For a **saved** network, pass `None`. NetworkManager already has the password.
- For a **new secured** network, pass `Some(password)`. NetworkManager saves a new profile with it, so next time it's saved.
- For a **new open** network, pass `None`.

## A network list

```rust
use amane::{App, Column, LayerWindow, Network, Parent, Rectangle, Service, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let network = Network::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for access_point in network.access_points() {
        let mark = if access_point.active() { "* " } else { "" };

        let label = format!("{mark}{}  {}%", access_point.ssid(), access_point.strength());

        let ssid = String::from(access_point.ssid());

        rows.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(30.0)
                .on_click(move |_| Network::connect(&ssid, None))
                .child(Text::new(label)),
        ));
    }

    LayerWindow::new().width(300.0).height(400.0).child(Column::new(rows).width(Parent))
}
```

`ssid` is copied into a `String` before the closure, because the closure lives longer than `network` ([Input](input.md)).

For new secured networks, pair this with a password `TextInput` ([Text Input](text_input.md)).
