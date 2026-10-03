# Views

A bar shows data that changes: the time, the battery, the volume. Most UI toolkits build the widgets once, and you write code to update each widget when its data changes. Amane works differently. A view is a function that describes what a window shows, and Amane calls it again whenever that should change, so there's no update code to write.

This page explains when a view runs, and the one rule that follows from that.

## A view with live data

```rust
use amane::{App, Battery, Full, LayerWindow, Service, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let battery = Battery::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Text::new(format!("battery {}%", battery.percent())))
}
```

When the battery goes from 80% to 79%, the text changes on its own.

## When a view runs

`view` is called again every time the window needs to change:

1. Amane calls `view`.
2. `view` reads the battery (80%) and builds a new window with a `Text` that says 80%.
3. Amane draws it, then throws the widgets away.
4. The battery drops to 79%.
5. Amane calls `view` again. It reads 79% and builds a new `Text` that says 79%.

The first draw and every update go through the same function, so the screen always matches the data.

Amane also knows *which* windows to call again. `Battery::read()` records that this view read the battery. When the battery changes, only the windows whose view read it are drawn again. A window that never reads the battery is left alone. [State and Services](services.md) explains this in detail.

## Keeping state outside the view

The widgets are thrown away after every frame. So:

**Nothing stored in a widget survives to the next frame.**

Anything that has to last, like a counter, whether a menu is open, or text the user typed, has to live outside the view. Amane gives you three places for it:

- a **Service**, for your own state ([State and Services](services.md))
- a built-in name-keyed store, used by `ScrollArea` and `TextInput` to keep their scroll position and text between frames. You give each one a name, like `ScrollArea::new("apps", ...)`.
- a `static` you manage yourself, like a `thread_local!` or a `Mutex`. The window you click or type in always redraws after the event, so a static changed by its own input handler shows up. But a static changed anywhere else, like from a thread or another window's handler, redraws nothing. A Service redraws every window that reads it, wherever the change comes from.

## What a view should and shouldn't do

A view runs often, sometimes 60 times a second during an animation, on the same thread that draws every window. So a view should:

- **read** data (`SomeService::read()`) and **build** widgets
- **not** wait on anything: no running commands, no reading files, no network calls, no `sleep`

Slow work goes in a Service's own thread instead. The view then reads the result.

## Splitting a view into functions

`App::window` takes `fn() -> LayerWindow`: a function that takes nothing and returns a window. It can't be a closure that captures variables. That's on purpose: since a view can't capture anything, everything it shows has to come from somewhere Amane can watch.

You can split a view into helper functions freely:

```rust
use amane::{App, Color, Full, LayerWindow, Parent, Rectangle, Row, Text, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Row::new(children![label("left"), label("right")]))
}

fn label(content: &str) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::BLACK)
        .child(Text::new(content).color(Color::WHITE))
}
```

Helpers can take arguments and return any widget type. Only the top-level view has a fixed signature.

## Terms

- **View:** a function that builds a window's widgets from the current data.
- **Frame:** one run of the view, plus drawing the result.
- **Immediate mode:** this style of UI, where the widgets are built again for every frame instead of being kept and updated.
