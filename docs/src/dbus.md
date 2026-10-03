# D-Bus

D-Bus is how most Linux system services talk: NetworkManager, UPower, BlueZ, logind, media players, notification daemons. Amane's built-in Services use it, and you can use the same small wrapper for anything they don't cover.

## The two buses

```rust,ignore
let bus = Bus::system();    // shared by every user: NetworkManager, UPower, logind, BlueZ
let bus = Bus::session();   // your own programs: media players, notifications, the tray
```

There's one connection per bus for the whole shell, shared by every thread. If a bus can't be reached, every call on it answers with nothing instead of failing. A bus is only tried once, so a bus that starts after your shell needs a shell restart.

## Reading a property

```rust
use std::time::Duration;

use amane::{App, Bus, Full, LayerWindow, Service, Text};

struct Wifi {
    enabled: bool,
}

impl Service for Wifi {
    fn new() -> Self {
        Self { enabled: read_enabled() }
    }

    fn interval() -> Duration {
        Duration::from_secs(5)
    }

    fn update(&mut self) -> bool {
        let enabled = read_enabled();

        let changed = enabled != self.enabled;

        self.enabled = enabled;

        changed
    }
}

fn read_enabled() -> bool {
    let bus = Bus::system();

    let enabled = bus.property(
        "org.freedesktop.NetworkManager",    // who to ask
        "/org/freedesktop/NetworkManager",   // which object
        "org.freedesktop.NetworkManager",    // which interface
        "WirelessEnabled",                   // which property
    );

    enabled.bool()
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let wifi = Wifi::read();

    let label = if wifi.enabled { "wifi on" } else { "wifi off" };

    LayerWindow::new().width(Full).height(30.0).child(Text::new(label))
}
```

To find names to use, explore the bus with a tool like `busctl`, `qdbusviewer`, or D-Spy.

D-Bus calls wait for an answer, so make them in a Service's thread, not in a view.

## Values

Every answer is a `Value`:

| Variant | Read it with |
|---|---|
| `Value::Bool` | `.bool()` |
| `Value::Number` | `.number()`, as an `f64`, whatever number type D-Bus used |
| `Value::Text` | `.text()` |
| `Value::List` | `.list()` |
| `Value::Map` | `.get("key")` |
| `Value::Nothing` | the call failed, or there was no answer |

Each reader gives an empty answer for the wrong kind: `false`, `0.0`, `""`, an empty list, or `Value::Nothing`. So a call to a program that isn't running gives `Value::Nothing`, and reading it shows nothing instead of crashing. Readers chain:

```rust,ignore
let title = metadata.get("xesam:title").text();
let first_artist = metadata.get("xesam:artist").list().first().map(|artist| artist.text());
```

## Calling methods

```rust,ignore
let answer = bus.call(destination, path, interface, method, &arguments![...]);
```

`arguments!` turns plain Rust values into D-Bus arguments: `bool`, `i32`, `u32`, `i64`, `f64`, `&str`, `String`, and `Vec<String>`. For other D-Bus types, build an `Argument` directly, like `Argument::Path(String::from("/org/..."))` for an object path, or `Argument::Variant(...)`.

Pause Spotify through MPRIS:

```rust,ignore
let bus = Bus::session();

bus.call(
    "org.mpris.MediaPlayer2.spotify",
    "/org/mpris/MediaPlayer2",
    "org.mpris.MediaPlayer2.Player",
    "Pause",
    &[],
);
```

Most methods answer with one value, which comes back on its own. A method that answers with several values gives a `Value::List`.

To set a property:

```rust,ignore
bus.set_property(destination, path, interface, "Powered", Argument::from(true));
```

## Listening for signals

Programs announce changes with signals. `signals` waits for each one, so call it in a Service's `listen`:

```rust,ignore
fn listen() {
    let bus = Bus::system();

    for signal in bus.signals("org.freedesktop.DBus.Properties", "PropertiesChanged") {
        if signal.path() != "/org/freedesktop/NetworkManager" {
            continue;
        }

        let enabled = read_enabled();

        Self::write().enabled = enabled;
    }
}
```

Each `Signal` has `sender()`, `path()`, and `arguments()`. `sender()` is the sender's unique name, like `:1.42`, not a well-known name like `org.mpris.MediaPlayer2.mpv`.

## Answering calls

Your shell can offer its own D-Bus interface, so other programs can call it:

```rust
use amane::{App, Bus, LayerWindow, Service, Text, arguments};

struct Server;

impl Service for Server {
    fn new() -> Self {
        Server
    }

    fn listen() {
        let bus = Bus::session();

        // watch first, then take the name, so the first calls aren't missed
        let methods = bus.methods("/org/example/Shell", "org.example.Shell");

        if !bus.own("org.example.Shell") {
            return;
        }

        for method in methods {
            if method.name() == "Ping" {
                method.reply(&arguments!["pong"]);
            }
        }
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    // reading the Service once starts its thread
    let _server = Server::read();

    LayerWindow::new().width(200.0).height(30.0).child(Text::new("serving"))
}
```

Test it with:

```sh
busctl --user call org.example.Shell /org/example/Shell org.example.Shell Ping
```

- `own(name)` takes a well-known name, and returns `false` if another program already has it. Amane never takes a name away from another program.
- `methods(path, interface)` waits for each call to that object and interface.
- `method.arguments()` gives the call's arguments, and `method.reply(...)` answers it.
- `bus.emit(path, interface, name, &arguments![...])` sends a signal of your own.
