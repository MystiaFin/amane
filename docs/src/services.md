# State and Services

A view is rebuilt on every frame ([Views](views.md)), so it can't hold data. A Service is where data lives instead. It keeps a value between frames, updates it in the background, and redraws the windows that show it when it changes.

This page shows how to read a Service, how to write your own, and the rules for using them.

## A clock

```rust
use std::time::Duration;

use amane::{App, Full, LayerWindow, Service, Text};

struct Clock {
    time: String,
}

impl Service for Clock {
    fn new() -> Self {
        Self { time: amane::output("date +%H:%M:%S") }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let time = amane::output("date +%H:%M:%S");

        let changed = time != self.time;

        self.time = time;

        changed
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let clock = Clock::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Text::new(&clock.time))
}
```

`std::time::Duration` comes from Rust's standard library. `amane::output` runs a shell command and returns what it printed ([Commands and Files](processes.md)).

## How a Service updates

1. The first time anything calls `Clock::read()`, Amane creates the clock with `new()` and keeps it for as long as the shell runs. There's only ever one `Clock`.
2. Amane also starts a background thread just for the clock. Every `interval()`, that thread calls `update()`.
3. `update()` returns whether anything the windows show has changed. When it returns `true`, Amane redraws every window whose view read the clock.
4. `view` calls `Clock::read()`, which gives it the current value and records that this window depends on the clock.

The clock's thread does the waiting, so the bar never freezes, even if `date` were slow.

## Reading a Service

```rust,ignore
let clock = Clock::read();
```

`read()` works from anywhere: views, input handlers, other Services. In a view it also subscribes the window to that Service, which is what makes it redraw on changes.

The value it returns locks the Service for reading. Keep it for as long as you need it, then let it go. In a view, that's usually until the end of the function.

## Writing to a Service

```rust,ignore
Counter::write().count += 1;
```

`write()` gives you the Service to change. When the value it returns goes away (at the end of the statement here), Amane marks the Service as changed and redraws every window that reads it. You don't call anything to redraw.

Write from input handlers ([Input](input.md)), from IPC handlers ([IPC](ipc.md)), or from the Service's own thread.

**Never call `write()` inside a view.** The view may still be holding a `read()` of the same Service, and a write waits for every read to finish, so the shell freezes forever. A view only reads.

## State that only changes through input

Many Services don't poll anything. A menu's open state, a counter, or a selected tab only change when you click. Give those a long interval, and leave out `update`:

```rust
use std::time::Duration;

use amane::{App, Button, LayerWindow, Parent, Rectangle, Service, Text};

struct Counter {
    count: i32,
}

impl Service for Counter {
    fn new() -> Self {
        Self { count: 0 }
    }

    // nothing changes on its own, only through clicks
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let counter = Counter::read();

    LayerWindow::new().width(200.0).height(40.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill("#1e1e2e")
            .on_click(clicked)
            .child(Text::new(format!("clicked {} times", counter.count))),
    )
}

fn clicked(_: Button) {
    Counter::write().count += 1;
}
```

`update` defaults to "nothing changed", so polling it costs nothing.

## Listening instead of polling

Some data announces its own changes: a command that streams events, a file being saved, a D-Bus signal. Polling those wastes time and adds delay. Replace `listen` instead:

```rust
use amane::{App, Full, LayerWindow, Service, Text};

struct Niri {
    event: String,
}

impl Service for Niri {
    fn new() -> Self {
        Self { event: String::from("waiting for niri") }
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

    LayerWindow::new().width(Full).height(30.0).child(Text::new(&niri.event))
}
```

`listen` runs on the Service's own thread, so it can wait as long as it likes. The default `listen` is the polling loop from [How a Service updates](#how-a-service-updates). Replacing it means `interval` and `update` aren't used anymore.

If `listen` panics, for example because a program it talks to went away, Amane prints a message and starts it again after 5 seconds. If `listen` returns, the Service keeps its last value and stops updating.

## Several Services in one view

A view can read as many Services as it likes:

```rust,ignore
let battery = Battery::read();
let audio = Audio::read();
let clock = Clock::read();
```

The window redraws when any of them changes. A window that reads none of them is never redrawn because of them. That's how a shell with many windows stays cheap: each window only wakes up for its own data.

## Built-in Services

Amane comes with Services for common system data: `Battery`, `Audio`, `Network`, `Media`, `Notifications`, `Workspaces`, and more. You use them exactly like your own, with `read()`, and they also have functions to control them, like `Audio::set_volume(50)`. See [Built-in Services](system.md).

## Terms

- **Service:** one global value per type, with its own background thread. Reading it in a view subscribes the window to it.
- **Poll:** asking for new data on a timer, with `interval` and `update`.
- **Listen:** waiting for data to arrive, by replacing `listen`.
