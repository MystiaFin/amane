# Architecture

This file explains how Amane is put together: the big idea, what happens in one frame, where each part lives, and the rules the code depends on. Read it before changing Amane's source.

It names modules and types, not line numbers, so it stays true as the code changes. To find something, search for the type name.

## The big idea

A desktop shell has to show live data: the time, the battery, the workspaces. The usual way is to build the widgets once, then write code that updates each one when its data changes. That update code is where most bugs live.

Amane doesn't update widgets. A view is a plain function that builds the whole window from scratch:

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

When something the window shows changes, Amane runs the view again and gets a new widget tree. When the frame is drawn, the old tree is dropped. The first draw and every update go through the same code, so the screen can't drift out of sync with the data.

This has one big effect: **nothing stored in a widget survives to the next frame.** Anything that has to last lives outside the tree:

- in a **Service**, a global piece of state per type, like `Battery` or your own `Counter`
- in a small table keyed by a name, like the scroll position of `ScrollArea("my_list", ...)`
- in the backend's `OpenWindow`, for what only the backend needs (size, scale, the settings last sent)

## Startup

1. `App::new()` collects the view functions: `window`, `window_per_monitor`, `normal_window`, `lock`, plus the IPC handlers.
2. `App::run()` checks that at least one window was set, tunes glibc's allocator (`allocator::limit`), and sets the default font.
3. It hands everything to `WaylandApp::new` in `src/wayland.rs`. That function connects to the compositor, binds the protocols Amane needs, and builds `WaylandState`.
4. Windows are opened. Per-monitor windows wait until the compositor has described each monitor.
5. `WaylandApp::run` starts the event loop, which runs until the program exits.

## The event loop

Everything that touches Wayland or draws runs on **one thread**, inside one calloop event loop with three sources:

- **the Wayland queue:** messages from the compositor, like configure, frame callbacks, and pointer and key events
- **the IPC socket:** `amane ipc call <name>` from the command line
- **the wake ping:** a signal other threads send when something changed

The loop sleeps until one of them is ready. Then it runs that source's callback with `&mut WaylandState`. SCTK turns compositor messages into calls on handler traits, like `LayerShellHandler::configure`, which are wired up with `delegate_dispatch2!`.

## One frame

A frame always goes through the same four steps, in `redraw` (`src/wayland/redraw.rs`) and `src/frame.rs`:

1. **Run the view.** `frame::run_view` clears the list of Services read so far, runs the view, and returns the widget tree plus the set of Services it read.
2. **Update the window.** `update_surface` compares the new `LayerWindow` settings (size, anchor, layer, margin, keyboard mode, input area) with the ones last sent, and sends only what changed.
3. **Lay out and record.** `frame::build` places the tree in the window and asks every widget to draw into a `Renderer`. Nothing is drawn yet. The renderer only records a list of `Command`s. It then collects the input targets.
4. **Present.** The `Gpu` turns the commands into pixels and shows them.

Only the first frame comes from the compositor's configure message. After that, a window redraws only when something asks it to.

## What makes a window redraw

There are three causes, and they all end in `redraw`:

- **A Service changed.**
  1. When a view calls `SomeService::read()`, it notes the Service's `TypeId`.
  2. When a `Write` guard is dropped, it calls `changes::mark`, which records the change and pings the event loop.
  3. The loop redraws only the windows whose last view read that Service. It checks this with `is_disjoint`.
- **An animation is still moving.** Reading `Animation::value()` before the animation has finished raises one global flag (`moving`). After drawing, `redraw` sees the flag and asks the compositor for the next frame callback. When every animation has arrived, nothing raises the flag and drawing stops.
- **Something asked directly.** `request_frame()`, opening or closing a window, and locking all go through `changes::mark_all` or a queue that the wake callback reads.

Frame requests wait for the compositor's frame callback, so several changes inside one display refresh become one frame.

## Widgets and layout

Every widget implements the same trait, `Widget` (`src/widgets.rs`):

- `width()` and `height()` say how big it wants to be, as a `Size`: a fixed number of pixels, or `Parent` ("as big as you let me").
- `draw()` records its drawing into the `Renderer`.
- `collect_targets()` adds its clickable areas.

Containers hold their children as `Box<dyn Widget>`. Layout runs in two passes:

1. **Measure, going up.** Children say what they need.
2. **Place, going down.** The parent hands each child a real area.

`Row` and `Column` share one engine, `Layout` (`src/widgets/layout.rs`), which only differs in direction.

Builders that need a value before they're usable use typestate. For example, `Rectangle::new()` returns a `RectangleNeedsWidth`, so forgetting a size is a compile error, not a blank window.

## Drawing

Drawing is split into two halves, so no code outside the GPU module knows how pixels are made:

- **`Renderer`** (`src/graphics/renderer.rs`) is a recorder: a list of `Command`s plus the current transform. Widgets call methods like `rectangle`, `border` and `text`. Each one pushes one command, or none if nothing would show.
- **`Gpu`** (`src/graphics/gpu.rs`) draws the list onto a canvas, then copies the canvas into the window. Each command goes to the cheapest engine that can draw it:
  - Amane's own quad pipeline for rectangles and letters
  - vello for everything else
  - the order is kept by drawing one engine's collected work before switching to the other

All windows share one graphics device. Vello work that didn't change since the last frame is reused.

If the GPU library is ever replaced, no widget has to change.

## Input

Each frame saves a list of targets. A target is an area, its handlers, and the inverse of its transform. Parents come before their children in the list.

- **Mouse:** an event finds the **last** target that contains the point, which is the innermost, topmost widget, and calls its handler.
- **Keys:** keys go to the focused `TextInput` first (`src/input/focus.rs`), then to the window's `on_key`.

Handlers are stored as `Rc`, so they outlive the tree they came from. That tree is gone by the time the click arrives.

## Services

A Service is one global value per type (`src/services.rs`, `src/services/store.rs`):

- It's created the first time it's used, and kept until the program exits.
- Each Service gets its own background thread running `listen()`.
  - The default `listen` polls `update()` every `interval()`.
  - Event-driven Services, like D-Bus and PulseAudio ones, replace it with their own loop.
  - If `listen` panics, it starts again after 5 seconds.
- Control calls from input handlers, like setting the volume, run on one shared worker thread (`src/services/worker.rs`), so a slow bus never stalls drawing.
- A poll that found nothing new marks its write as `quiet`, so it wakes no window.

`Workspaces` doesn't know which compositor is running. `src/compositor.rs` picks a backend from the environment (`NIRI_SOCKET`, `HYPRLAND_INSTANCE_SIGNATURE`, `SWAYSOCK`), and every backend hands over the full workspace list after each event:

- **niri** only sends what changed, so its backend remembers the rest and rebuilds the list itself.
- **Hyprland** and **Sway** are asked for the full list again after every event.

The Service only sorts the list and stores it. Adding a compositor means one file in `src/compositor/` and one arm in each `match` in `compositor.rs`.

## Code map

```
src/
├── lib.rs             the only public gate: private mods + pub use
├── app.rs             App: collects views and handlers, then starts the backend
├── frame.rs           run_view and build: one frame, minus Wayland
├── changes.rs         which Services changed, which were read, the wake ping
├── animation.rs       Animation, Easing, and the moving flag
├── placement.rs       Size, Align, Padding
├── layer_window.rs    LayerWindow and its settings types
├── window.rs          Window (normal windows), window_size, open/close
├── monitor.rs         Monitor
├── wayland.rs         WaylandState, WaylandApp, the connection
├── wayland/
│   ├── surface.rs     OpenWindow, Role, View, Content
│   ├── windows.rs     opening and adding windows
│   ├── layer.rs       layer settings and LayerShellHandler
│   ├── redraw.rs      resize, redraw, present
│   ├── update.rs      sending changed settings and input regions
│   ├── pointer.rs     buttons, scroll, cursor
│   ├── keyboard.rs    keys
│   ├── output.rs      monitors coming and going
│   ├── normal.rs      xdg windows
│   ├── lock.rs        the session lock surfaces
│   ├── scale.rs       fractional scale
│   ├── socket.rs      the IPC socket source
│   └── wake.rs        the wake ping source
├── widgets.rs         the Widget trait
├── widgets/           Rectangle, Text, TextInput, Row, Column, Stack, ScrollArea, Canvas, shapes
├── graphics.rs        Color, Rect, Transform, Gradient, BezierPath
├── graphics/
│   ├── renderer.rs    Renderer and Command
│   ├── gpu.rs         Gpu and everything under gpu/
│   ├── font.rs        font loading through fontconfig
│   └── image.rs       PNG, JPEG and SVG decoding
├── input.rs           Button, Key, Scroll, Cursor
├── input/             targets, pointer state, keyboard focus
├── services.rs        the Service trait
├── services/          the built-in Services, the store, Write, the worker
├── style/             Fill, Radius, Shadow, Image, Mask
├── dbus.rs, dbus/     a small zbus wrapper: Bus, Method, Signal, Value
├── ipc.rs             IpcHandlers and IpcCall
├── compositor.rs      picks the workspace backend for the running compositor
├── compositor/        niri, Hyprland and Sway IPC, used by Workspaces
├── process.rs         spawn, output, lines
├── files.rs           watch_file, through inotify
├── allocator.rs       glibc allocator limits for image decoding
└── timing.rs          AMANE_FRAMES frame logging
cli/                   the amane command: startup, dev, compile, run, ipc call
examples/              one small program per feature
```

## Invariants

These rules aren't checked by the compiler. Breaking one causes a crash, a freeze, or a window that stops updating.

- **Struct field order is drop order.** Rust drops fields from top to bottom, and some fields must die before others:
  - In `OpenWindow`, `gpu` comes before `fractional`, which comes before `role`. The GPU draws into the surface, and the fractional scale object belongs to that surface.
  - In `WaylandState`, `windows` comes before `connection`.
  - In `WaylandApp`, `state` comes before `event_loop`, because the event loop holds the connection the GPU draws through.

  Don't reorder these fields.
- **Wayland and drawing stay on the main thread.** Other threads never touch `WaylandState`. They mark a change and ping (`changes::mark`, `changes::mark_all`), and the event loop does the work.
- **Widgets keep no state between frames.** Lasting state goes in a Service or a name-keyed table.
- **Views read Services only through `read()`.** `read()` is what records the dependency. Data reached any other way won't redraw the window when it changes.
- **Never call `write()` inside a view.** The view may still hold a read of the same Service, so the write waits forever. Write from input handlers, or from the Service's own thread.
- **The last matching target wins.** Targets are collected parent first, so the hit test must search from the end.
- **Slow work never runs on the main thread.** Commands, file watching, image decoding, PAM and D-Bus calls all run on other threads.

## Cross-cutting concerns

**Errors**
- Setup failures, like a missing Wayland protocol, panic with `expect("failed to <verb> <thing>")`. A shell that can't start should say why and stop.
- Runtime failures don't panic:
  - a D-Bus call that fails returns `Value::Nothing`
  - a Service whose `listen` panics restarts
  - a worker job that panics loses only that one call

**Debugging**
- Set `AMANE_FRAMES=1` to print one line per frame. It shows the time since the last frame, plus the view, draw and GPU times, and which Service woke the window.

**Testing**
- `src/frame.rs` has layout tests. Each one builds a small tree and checks exactly where each widget lands. Add one when you change layout.
- Each compositor backend has a parse test fed with trimmed replies from that compositor. Add one when you add a backend.
- Only library tests exist, and they need no compositor. CI runs them on every push (`.github/workflows/test.yml`): `nix develop -c cargo test --lib`.

**Module layout**
- A module is `foo.rs` plus an optional `foo/` folder, never `mod.rs`.
- `lib.rs` holds only private `mod` lines and `pub use` lines, so the public API is the list in that one file.

## Terms

- **Compositor:** the program that owns the screen, like niri. Amane is a client of it.
- **Layer surface:** a window that's part of the desktop, like a bar or a wallpaper (wlr-layer-shell).
- **Configure:** the compositor's message that a window may draw, and at what size.
- **Frame callback:** the compositor's signal that now is a good time to draw the next frame.
- **Immediate mode:** building the UI from scratch every frame instead of updating it.
- **Service:** a global, typed piece of state with its own thread. Reading it subscribes the window to its changes.
- **Target:** an area that reacts to the mouse, saved from the last frame.
- **Typestate:** a builder whose type changes as required values are given, so a missing value is a compile error.
