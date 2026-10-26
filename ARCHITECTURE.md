# Architecture

This file explains how Amane works: the general idea, what happens in one frame, where each part lives, and the rules the code depends on.
If you want to contribute, I recommend reading this first.

I name modules and types here instead of line numbers, so this file doesn't go out of date every time the code moves.
If you want to find something, search for the type name.

## The general idea

A desktop shell always has to show live data, like the time, the battery, or which workspace you're on.
Most UI toolkits build the widgets once and keep them in memory.
When the data changes, you write code that finds the right widget and updates it.
This is called *retained mode*.

The catch is that you now have two copies of the truth: what the backend knows, and what the widget remembers.
You have to keep them in sync yourself, and that sync code is where most bugs live.

Amane's widgets don't remember *anything*(tho, it will be in the future. cus I'm planning a hybrid retained mode later.).
A view is a plain function that builds the whole window from scratch:

```rust
use amane::{Battery, Full, LayerWindow, Service, Text};

fn view() -> LayerWindow {
    let battery = Battery::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Text::new(format!("{}%", battery.percent())))
}
```

This is called *immediate mode*.
When something a window shows changes, Amane runs that window's view again, draws the new tree, and throws the old one away.
The first draw and every update go through the same code, so the screen can't drift away from the data.

So nothing you put in a widget survives to the next frame.
Anything that has to last lives in a `Service` (like `Battery`, or your own `Counter`), in a small table keyed by a name (like the scroll position of `ScrollArea::new("my_list", ...)`), or in the backend's `OpenWindow`.

I know rebuilding everything sounds expensive.
But this is a desktop shell, not a browser.
Your widget tree is small, and building it costs very little next to drawing it.

I came to this after writing a Quickshell config.
My volume slider was jaggy because it waited for the backend to send the new volume back, so I had to keep the dragged value and the real one from fighting each other.
In Amane, the dragged value is just a variable your view reads.
There's no widget holding a second copy that you have to keep in sync.

## Startup

`App::new()` collects your view functions (`window`, `window_per_monitor`, `normal_window` and `lock`) and your IPC handlers.
Nothing happens until `App::run()`, which hands everything to `WaylandApp::new` in `src/wayland.rs`.

`WaylandApp::new` connects to the compositor and binds the protocols Amane needs, through smithay-client-toolkit (SCTK).
The compositor, wlr-layer-shell, xdg-shell and shm are required, so a missing one is a panic.
Everything else is optional, and Amane works without it.

Then the windows open.
Per-monitor windows wait until the compositor has described each monitor.
Nothing is drawn until the compositor sends the first configure.

## The event loop

Everything that touches Wayland or draws runs on one thread, inside one calloop event loop.
The loop has three sources: the Wayland queue, the IPC socket, and a wake ping that other threads use to say something changed.

The loop sleeps until one of them is ready, then runs that source's callback with `&mut WaylandState`.

Two Wayland words come up a lot in the rest of this file.
A *configure* is the compositor telling a window it may draw, and at what size.
A *frame callback* is the compositor saying "now is a good time to draw the next frame".

## One frame

A frame is `OpenWindow::redraw` in `src/wayland/redraw.rs`, and it goes through the same steps every time.

1. `frame::run_view` runs your view and returns the new `LayerWindow`, together with the Services the view read.
2. `update_surface` compares the window settings the view asked for with the ones it last sent, and only sends what changed. If the window is hidden or not configured yet, the frame stops here.
3. `frame::build` lays the tree out and asks every widget to draw into a `Renderer`, which only records a list of `Command`s. It also collects the input targets for this frame.
4. The `Gpu` turns the commands into pixels and presents them.

If anything was still animating, the window asks for a frame callback in the same commit, so the next frame arrives on the next refresh.

Views, layout, `window_size()` and `Monitor` all work in widget units, before `App::scale_factor` is applied.

## What makes a window redraw

There are a few ways, but they all end up calling `redraw`.

The compositor can cause one directly, with a configure or a change of scale.

A Service can change.
When a view calls `SomeService::read()`, the Service is noted.
When a `Write` guard is dropped, it calls `changes::mark` and pings the event loop.
Then only the windows whose last frame read that Service are redrawn.
A write that changed nothing can be marked `quiet`, and then it wakes nobody.

Some things affect every window: `open_window`, `close_window`, `Lock::start` and IPC calls all call `changes::mark_all`.

Input redraws only the window it happened in.
If the handler wrote a Service, that write wakes the Service's readers in other windows on its own.

Animations keep going until they're done.
Reading `Animation::value()` before it has arrived, or calling `request_frame()`, asks for the next frame.

`OpenWindow::request_frame` is where these meet.
If a frame is already requested, it does nothing, so several changes inside one display refresh become one frame.

## Widgets and layout

Every widget implements the `Widget` trait in `src/widgets.rs`.
`width()` and `height()` say how big it wants to be, `draw()` records its drawing, and `collect_targets()` adds the areas where it reacts to the pointer.

`Row` and `Column` share one engine, `Layout` in `src/widgets/layout.rs`, and the only difference between them is the direction.
Fixed children get their own size, `Parent` children split whatever is left evenly, and then `justify` and `align` decide where everything sits.

Some builders need a value before they can be used.
`Rectangle::new()` returns a `RectangleNeedsWidth`, `.width()` turns it into a `RectangleNeedsHeight`, and only `.height()` gives you a real `Rectangle`.
So forgetting a size is a compile error instead of a blank window.

## Drawing

Drawing is split in two, so no code outside `src/graphics/gpu/` knows how pixels are made.

The `Renderer` in `src/graphics/renderer.rs` is a recorder.
Widgets call methods like `rectangle`, `border` and `text`, and each one pushes one command.

The `Gpu` in `src/graphics/gpu.rs` draws that list.
Every window has its own `Gpu`, but they all share one GPU device, so the driver doesn't switch contexts and shaders compile only once.

Rectangles, borders, letters and images go to Amane's own quad pipeline (`quads.wgsl`), because that's most of what a shell draws and plain triangles are fast.
Everything else (paths, strokes, gradients, shadows) goes to vello.
To keep the order right, whichever engine was collecting work draws it before the other one takes over.

Blur, cut and custom shaders work on what the canvas already holds, so everything before them gets drawn first.

## Input

Each frame saves a list of targets: an area, its handlers, and the inverse of its transform, so a rotated widget still gets hit in the right place.
Targets are collected parents first, so the last target that contains a point is the innermost one.

A click only counts when the button is pressed and released over the same target.
A drag sticks to the target where it started.

Keys go to the focused `TextInput` first (`src/input/focus.rs`), and anything it doesn't take goes to the window's `on_key`.

## Services

A Service is one global value per type, defined in `src/services.rs`.

It's created the first time anything reads or writes it, and it gets its own background thread that runs `listen()`.
The default `listen` calls `update()` every `interval()`.
Event-driven Services, like the D-Bus and PulseAudio ones, replace it with their own loop.

Control calls from input handlers, like `Audio::set_volume`, go to one shared worker thread (`src/services/worker.rs`), so a slow bus never stalls drawing.

`Workspaces` doesn't know which compositor is running.
`src/compositor.rs` picks a backend from whichever of `NIRI_SOCKET`, `HYPRLAND_INSTANCE_SIGNATURE`, `SWAYSOCK` or `MANGO_INSTANCE_SIGNATURE` is set.

To add a compositor, you add one file in `src/compositor/`, its environment variable in `running()`, and one arm in each of the two `match`es (`listen` and `focus_workspace`).

## The amane CLI

The `amane` command in `cli/` builds and runs your shell.
It carries a copy of `src/` inside itself and unpacks it into `~/.cache/amane/library`, so your shell always builds against the library that came with your `amane` binary.

`amane dev` rebuilds on every save, and keeps the old shell running while a build fails.

## Where things live

```
src/
├── lib.rs             the only public gate: private mods + pub use
├── app.rs             App: collects views and handlers, then starts the backend
├── frame.rs           run_view and build: one frame, minus Wayland
├── changes.rs         which Services changed, which were read, the wake ping
├── animation.rs       Animation, Easing, request_frame and the moving flag
├── placement.rs       Size, Align, Justify, Padding
├── full.rs            Full, for a full-width window or a fully round corner
├── layer_window.rs    LayerWindow and its settings types
├── window.rs          Window (normal windows), window_size, open/close
├── monitor.rs         Monitor
├── wayland.rs         WaylandState, WaylandApp, the connection
├── wayland/
│   ├── surface.rs     OpenWindow, Role, View, Content
│   ├── windows.rs     opening, finding and closing windows, redraw requests
│   ├── compositor.rs  frame callbacks and integer scale changes
│   ├── layer.rs       layer settings and LayerShellHandler
│   ├── redraw.rs      redraw, request_frame, present
│   ├── update.rs      sending changed settings and input regions
│   ├── seat.rs        mice and keyboards being plugged in and out
│   ├── pointer.rs     buttons, scroll, cursor
│   ├── keyboard.rs    keys
│   ├── dropped.rs     files dragged in from other programs
│   ├── output.rs      monitors coming and going
│   ├── normal.rs      xdg windows
│   ├── lock.rs        the session lock surfaces
│   ├── scale.rs       fractional scale
│   ├── socket.rs      the IPC socket source
│   └── wake.rs        the wake ping source
├── widgets.rs         the Widget trait
├── widgets/           Rectangle, Text, TextInput, Row, Column, Stack, ScrollArea, Canvas, shapes
├── graphics.rs        Color, Area, Transform, Gradient, BezierPath
├── graphics/
│   ├── renderer.rs    Renderer and Command
│   ├── gpu.rs         Gpu and everything under gpu/
│   ├── font.rs        font loading through fontconfig
│   └── image.rs       PNG, JPEG and SVG decoding on a background thread
├── input.rs           Button, Key, Scroll, Cursor
├── input/             targets, pointer state, keyboard focus
├── services.rs        the Service trait
├── services/          the built-in Services, the store, Write, the worker
├── style.rs, style/   Fill, Mask, Radius, Shadow, Image
├── dbus.rs, dbus/     a small zbus wrapper: Bus, Method, Signal, Value
├── ipc.rs             IpcHandlers and IpcCall
├── compositor.rs      picks the workspace backend for the running compositor
├── compositor/        niri, Hyprland, Sway and Mango IPC, used by Workspaces
├── process.rs         spawn, output, lines
├── files.rs           watch_file, through inotify
├── allocator.rs       glibc allocator limits for image decoding
└── timing.rs          AMANE_FRAMES frame logging
cli/                   the amane command: new, target, dev, compile, run, restart, clean, ipc call
examples/              one small program per feature
```

## Rules the compiler won't check

Breaking any of these gives you a crash, a freeze, or a window that silently stops updating.

Struct field order is drop order.
Rust drops fields from top to bottom, and some things have to die before others.
In `OpenWindow`, `gpu` comes before `fractional`, which comes before `role`, because the GPU draws into the surface and the fractional scale object belongs to that surface.
In `WaylandState`, `windows` comes before `connection`.
In `WaylandApp`, `state` comes before `event_loop`, because the event loop holds the connection the GPU draws through.
**DO NOT reorder these fields.**

Wayland and drawing stay on the main thread.
Other threads never touch `WaylandState`.
They mark a change and ping (`changes::mark` or `changes::mark_all`), and the event loop does the actual work.

Widgets keep no state between frames.
Anything that has to last goes in a Service or a name-keyed table.

Views read Services only through `read()`, because `read()` is what records that the window depends on it.
Data you reach any other way won't redraw the window when it changes.

Never call `write()` inside a view.
The view may still be holding a read of the same Service, so the write waits forever.
Write from input handlers, or from the Service's own thread.

The last matching target wins.
Targets are collected parents first, so hit testing has to search from the end.

Slow work stays off the main thread.
Commands, file watching, image decoding, PAM and D-Bus calls all run on other threads.
There's one known exception: reading the file list after a drag-and-drop blocks the loop for a moment, and it's noted in `dropped.rs`.

## Errors, debugging and tests

If the shell can't start, like when a Wayland protocol it needs is missing, it panics with `expect("failed to <verb> <thing>")`.
Once it's running, failures don't take it down.
A D-Bus call that fails returns `Value::Nothing`, a Service whose `listen` panics restarts after 5 seconds, a worker job that panics loses only that call, and a GPU error only loses its frame.

Set `AMANE_FRAMES=1` to print one line per frame, with how long each step took and which Service woke the window.

The library tests don't need a compositor, so CI runs them on every push with `nix develop -c cargo test --lib`.
Add a layout test in `src/frame.rs` when you change layout, and a parse test when you add a compositor backend.
Anything that draws still needs a real compositor, so run an example too, like `cargo run --example bar`.
