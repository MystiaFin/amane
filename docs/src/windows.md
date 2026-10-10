# Multiple Windows and Monitors

A shell is rarely one window. This page covers several layer windows, one window per monitor, normal app-style windows, and opening and closing windows while the shell runs.

## Several windows

Call `.window` once for each window:

```rust
use amane::{App, Full, Horizontal, LayerWindow, Parent, Rectangle, Vertical};

fn main() {
    App::new().window(bar).window(corner).run();
}

fn bar() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .child(Rectangle::new().width(Parent).height(Parent).fill("#1e1e2e"))
}

fn corner() -> LayerWindow {
    LayerWindow::new()
        .width(200.0)
        .height(60.0)
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Right)
        .child(Rectangle::new().width(Parent).height(Parent).fill("#313244"))
}
```

Each window has its own view, and each one only redraws for the Services its own view reads. Windows added with `.window` go on whichever monitor the compositor picks, usually the focused one.

## One window per monitor

A bar should be on every monitor, including ones you plug in later:

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Monitor, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window_per_monitor(bar).run();
}

fn bar(monitor: &Monitor) -> LayerWindow {
    let label = format!("{} {}x{}", monitor.name, monitor.width, monitor.height);

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
                .child(Text::new(label).color(Color::WHITE)),
        )
}
```

The view takes a `&Monitor`, with the monitor's `name` (like `"DP-1"`), `width`, and `height`. Its dimensions use the same units as widgets, accounting for both the compositor's scale and [the app's scale factor](scaling.md). Amane opens one window for each monitor when it starts, opens a new one when a monitor is plugged in, and closes it when the monitor is unplugged.

Use the monitor to show different things on different screens:

```rust,ignore
if monitor.name == "eDP-1" {
    // the laptop screen
}
```

## Normal windows

A normal window is a regular app window, with a title bar, that the compositor places and tiles like any other app. Use one for a settings panel, or anything you'd want to move around:

```rust
use amane::{App, Center, Color, Parent, Rectangle, Text, Window};

fn main() {
    App::new().normal_window("settings", settings).run();
}

fn settings() -> Window {
    Window::new()
        .title("amane settings")
        .size(480.0, 320.0)
        .resizable(true)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#313244")
                .align_child(Center, Center)
                .child(Text::new("settings go here").color(Color::WHITE)),
        )
}
```

- `title` defaults to `"amane"`.
- `size` is the size it opens at, 640 by 480 by default. The compositor may give it another size, and a tiling compositor usually does.
- `resizable` lets the user resize it. It's `false` by default.
- `on_key` works the same as on a layer window. A normal window gets keyboard focus like any app, so it needs no `Keyboard` setting.

The name, `"settings"` here, tells windows apart. It's used to close the window later.

## Opening and closing windows later

Windows given to `App` open when the shell starts. To open one later, for example from a button, call `open_window` with a name and a view:

```rust
use amane::{App, Full, LayerWindow, Parent, Pointer, Rectangle, Window, close_window, open_window};

fn main() {
    App::new().window(bar).run();
}

fn bar() -> LayerWindow {
    LayerWindow::new().width(Full).height(30.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill("#1e1e2e")
            .cursor(Pointer)
            .on_click(|_| open_window("settings", settings)),
    )
}

fn settings() -> Window {
    Window::new().title("settings").child(
        Rectangle::new()
            .width(120.0)
            .height(40.0)
            .fill("#f38ba8")
            .on_click(|_| close_window("settings")),
    )
}
```

- `open_window(name, view)` opens a normal window. If a window with that name is already open, it does nothing, so clicking the button twice doesn't open two.
- `close_window(name)` closes it. Closing a window that isn't open does nothing.
- The user can also close a normal window from its title bar or with their compositor's keybind.

Both work from anywhere: input handlers, IPC handlers, or a Service's thread.

`open_window` only opens normal windows. To show and hide a layer window, like a popup or a launcher, keep it open and use `.visible(...)` ([Layer Windows](layer_windows.md)).

## The real window size

A compositor can give a window another size than it asked for. Inside a view, `window_size()` gives the size the window really has, in widget units before [the app's scale factor](scaling.md):

```rust,ignore
let (width, height) = window_size();
```

It's `(0.0, 0.0)` until the compositor has said, which can happen for the very first frame.
