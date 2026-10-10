# Architecture

This file explains how Amane works: the general idea, what happens when the shell starts, what happens in one frame, where each part lives, and the rules the code depends on. If you want to contribute, I recommend reading this first.

I name modules and types here instead of line numbers, so this file doesn't go out of date every time the code moves. If you want to find something, search for the type name.

## The general idea

A desktop shell always has to show live data, like the time, the battery, or which workspace you're on. Most UI toolkits handle this by building the widgets once and keeping them in memory. When the data changes, you write code that finds the right widget and updates it. This is called *retained mode*, and it's done for performance, because nothing gets rebuilt that doesn't need to be.

The catch is that you now have two copies of the truth: what the backend knows, and what the widget remembers. You have to keep them in sync yourself, and that sync code is where most bugs live.

Amane's widgets don't remember *anything*. A view is a plain function that builds the whole window from scratch:

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

This is called *immediate mode*. When something a window shows changes, Amane runs that window's view again, gets a new widget tree, draws it, and throws the old one away. The first draw and every update after it go through the same code, so the screen can't drift away from the data.

This also means nothing you put in a widget survives to the next frame. Anything that has to last lives outside the tree, in one of three places. A `Service` is a global piece of state per type, like `Battery`, or your own `Counter`. A small table keyed by a name holds things like the scroll position of `ScrollArea::new("my_list", ...)` or the text inside a `TextInput`. And the backend's `OpenWindow` keeps what only the backend needs, like the window's size, its scale and the settings it last sent to the compositor.

I know rebuilding everything sounds expensive. But this is a desktop shell, not a browser. Your widget tree is small, and building it costs very little next to drawing it. You won't feel it.

I came to this after writing a Quickshell config. My volume slider was jaggy because it waited for the backend to send the new volume back, so I had to build an optimistic UI: the slider kept its own value while dragging, and I had to keep that value and the real one from fighting each other. Amane doesn't make PulseAudio answer any faster, since `Audio::set_volume` still waits for the sound server to announce the change. But if you want the slider to follow your finger, the dragged value is just a variable your view reads. There's no widget holding a second copy that you have to keep in sync.

## Startup

`App::new()` collects your view functions (`window`, `window_per_monitor`, `normal_window` and `lock`), plus your IPC handlers and an optional font. Nothing happens until `App::run()`.

`run()` first checks that you set at least one window and panics if you didn't. Then it calls `allocator::limit`, which tunes glibc's allocator so the memory used to decode big images goes back to the system afterwards instead of staying in the process. It sets the default font if you gave one, and hands everything to `WaylandApp::new` in `src/wayland.rs`. A `GpuSession` guard outlives the backend and releases the shared GPU cache after the windows are gone, on normal exit or startup failure. This happens before thread-local teardown, while wgpu's own thread-local state is still available.

`WaylandApp::new` connects to the compositor and binds the protocols Amane needs. Amane uses smithay-client-toolkit (SCTK) for this, so it doesn't have to write the Wayland boilerplate by hand. The compositor, wlr-layer-shell, xdg-shell and shm are required, so a missing one is a panic. Fractional scaling, the viewporter and drag-and-drop are optional, and Amane works without them.

Then the windows open. Per-monitor windows wait until the compositor has described each monitor. A layer window runs its view once at 0 by 0 before its surface even exists, because it needs the settings the view asks for (size, anchor, layer and so on) to create the surface. Nothing is drawn yet. Drawing starts when the compositor sends the first configure.

Finally `WaylandApp::run` starts the event loop. It runs until the last window closes, unless you have per-monitor windows, because those come back when a monitor is plugged in again. `App::quit` wakes the loop and asks it to stop, letting `App::run` return and release the windows. Apps listen on the session's IPC socket by default; `App::without_ipc` leaves it unused so a standalone app can run beside the shell.

## The event loop

Everything that touches Wayland or draws runs on one thread, inside one calloop event loop. The loop has three sources: the Wayland queue (messages from the compositor, like configures, frame callbacks and pointer and key events), the IPC socket (`amane ipc call <name>` from a terminal), and a wake ping that other threads use to say something changed.

The loop sleeps until one of them is ready, then runs that source's callback with `&mut WaylandState`. For the Wayland queue, SCTK turns each compositor message into a call on a handler trait, like `LayerShellHandler::configure` or `CompositorHandler::frame`. Those are wired up with `delegate_dispatch2!` at the bottom of `src/wayland.rs`.

Two Wayland words come up a lot in the rest of this file. A *configure* is the compositor telling a window it may draw, and at what size. A *frame callback* is the compositor saying "now is a good time to draw the next frame".

## One frame

A frame is `OpenWindow::redraw` in `src/wayland/redraw.rs`, and it goes through the same steps every time.

First it runs the view. `frame::run_view` clears the list of Services read so far, tells `window_size()` how big the window is, runs your view, and returns the new `LayerWindow` together with the set of Services the view read.

Then `update_surface` (in `src/wayland/update.rs`) compares the window settings the view asked for (size, anchor, layer, margin, keyboard mode, exclusive zone, visibility, input region) with the ones it last sent, and only sends what changed. Hiding a window takes its buffer away and sets its size to 0. If the window is hidden or not configured yet, the frame stops here.

Next, `frame::build` lays the tree out in the window and asks every widget to draw into a `Renderer`. Nothing is drawn yet at this point. The renderer only records a list of `Command`s. Some widgets read Services while drawing too (an image that is still decoding does this), so those reads are added to the window's list. After drawing, `build` collects the input targets, which replace the last frame's targets in the window's `Pointer`. The view's `on_key` replaces the old one the same way.

Last, the `Gpu` turns the commands into pixels and presents them, which attaches the buffer to the surface and commits it.

If anything in the frame was still animating, the window asks the compositor for a frame callback in that same commit, so the next frame arrives on the next refresh.

## What makes a window redraw

There are a few ways, but they all end up calling `redraw`, either right away or on the next frame callback.

The compositor can cause one directly. A configure resizes the window and redraws it, and so does a change of scale.

A Service can change. When a view calls `SomeService::read()`, the Service's `TypeId` is noted. When a `Write` guard is dropped, it calls `changes::mark`, which records which Service changed and pings the event loop. The wake callback then calls `request_changed_frames`, which only redraws the windows whose last frame read that Service (it checks with `is_disjoint`). A write that turned out to change nothing can be marked `quiet`, and then it wakes nobody.

Some things affect every window. `open_window`, `close_window` and `Lock::start` call `changes::mark_all`, and the wake callback opens, closes or locks whatever was asked for before redrawing. An IPC call also redraws every window, because a handler can change anything.

Input redraws only the window it happened in. When a pointer or key handler runs, that window asks for a frame. If the handler wrote a Service, that write wakes the Service's readers in other windows on its own, so hovering over your bar never redraws every window on every monitor.

A finished image wakes the windows waiting on it. Images decode on their own thread, and while one is decoding, the window that asked for it counts as having read a marker type called `Decoded`. When the decode is done it calls `changes::mark` with that type.

And animations keep going until they're done. Reading `Animation::value()` before the animation has arrived sets a global `moving` flag, and so does calling `request_frame()` from a view. After drawing, `redraw` sees the flag and asks for the next frame callback. When everything has arrived, nobody sets the flag anymore, and the window stops drawing.

`OpenWindow::request_frame` is where these meet. If a frame is already requested, it does nothing, so several changes inside one display refresh become one frame. The exception is a hidden or unconfigured window, which gets no frame callbacks at all, so it runs its view right away to find out whether it should show again.

## Widgets and layout

Every widget implements the `Widget` trait in `src/widgets.rs`. `width()` and `height()` say how big it wants to be, as a `Size`, which is either a fixed number of pixels or `Parent` ("as big as you let me"). `draw()` records its drawing into the `Renderer` inside the area it was given. `collect_targets()` adds the areas where it reacts to the pointer, and containers pass their children's areas along.

Containers hold their children as `Box<dyn Widget>`. `Row` and `Column` share one engine, `Layout` in `src/widgets/layout.rs`, and the only difference between them is the direction.

A `Layout` measures its children once, when it's built (in `Layout::new` and `set_gap`). That's fine because children never change after that, since the whole tree is rebuilt on the next frame anyway. If any child is `Parent`-sized in a direction, the layout is too. Placing happens later, during `draw`: fixed children get their own size, `Parent` children split whatever is left evenly, and then `justify` and `align` decide where everything sits.

Some builders need a value before they can be used, and those use *typestate*. `Rectangle::new()` returns a `RectangleNeedsWidth`, `.width()` turns it into a `RectangleNeedsHeight`, and only `.height()` gives you a real `Rectangle`. So forgetting a size is a compile error instead of a blank window. `Canvas` works the same way.

## Drawing

Drawing is split in two, so no code outside `src/graphics/gpu/` knows how pixels are made. If the GPU library ever gets replaced, no widget has to change.

The `Renderer` in `src/graphics/renderer.rs` is a recorder. It holds a list of `Command`s and the current transform, with the window's scale already in it. Widgets call methods like `rectangle`, `border` and `text`, and each one pushes one command, or none if nothing would show.

The `Gpu` in `src/graphics/gpu.rs` draws that list onto a canvas texture, then copies the canvas into the window. Every window has its own `Gpu`, but they all share one GPU device and one vello renderer, opened by the first window (`src/graphics/gpu/shared.rs`). Separate devices would be separate GPU contexts that the driver has to switch between, and each one would compile every shader again.

The `Gpu` walks the commands in `src/graphics/gpu/paint.rs` and sends each one to the cheapest engine that can draw it. Rectangles, borders, letters and images go to Amane's own quad pipeline (`quads.wgsl`), as long as they aren't rotated. That's most of what a shell draws, and plain triangles are much faster than vector shapes. Everything else (paths, strokes, gradients, shadows) goes to vello. To keep the order right, whichever engine was collecting work draws it before the other one takes over.

Blur, cut and custom shader commands are different, because they work on what the canvas already holds. So everything before them gets drawn first, and then they run. A faded group, or a clip that has one of those effects inside it, is drawn on its own canvas and laid down as one piece.

Vello costs about the same no matter how little it draws, so if a vello scene is exactly the same as last frame, its old result is laid down again instead (`painted.rs`). Scenes with images or gradients are never reused this way, because the textures they point at can change.

If the GPU is lost, like after a driver reset, Amane can't rebuild its textures yet, so it prints a message and exits instead of freezing.

## Input

Each frame saves a list of targets. A target is an area, its handlers, and the inverse of its transform, so a rotated or scaled widget still gets hit in the right place. Targets are collected parents first, so the last target that contains a point is the innermost one. The window's `Pointer` (`src/input/pointer.rs`) keeps the list until the next frame replaces it.

Clicks, scrolls, motion and the cursor shape all go to the last target under the pointer that has that kind of handler. A click only counts when the button is pressed and released over the same target. A drag sticks to the target where it started, even when the pointer leaves it. Hover is the exception: every hover target under the pointer is told when the pointer enters or leaves, not just the innermost one.

The handlers are `Rc`s, so they outlive the tree they came from. That tree is long gone by the time the click arrives.

The compositor tells Amane which window has keyboard focus. Keys go to the focused `TextInput` first (`src/input/focus.rs`), and only one text input in the whole shell can have focus at a time. Up, Down, Tab and keys Amane has no name for skip the text input. Escape goes to both, so the input loses focus and the window can still close itself. Anything the text input doesn't take goes to the window's `on_key`.

Files dragged in from a file manager are handled in `src/wayland/dropped.rs` and go to the target's `on_drop`.

## Services

A Service is one global value per type, defined in `src/services.rs` and stored by `src/services/store.rs`.

It's created the first time anything reads or writes it, and it stays until the program exits. When it's created, it also gets its own background thread that runs `listen()`. The default `listen` calls `update()` every `interval()`, and if `update` returns false, the write is marked quiet so it wakes no window. Event-driven Services, like the D-Bus and PulseAudio ones, replace `listen` with their own loop. Services that only change from input, like `Lock`, give it an empty one. If `listen` panics, like when a bus goes away, it starts again after 5 seconds.

Control calls from input handlers, like `Audio::set_volume`, don't run on the main thread. They go to one shared worker thread (`src/services/worker.rs`) that runs them in order, so a slow bus never stalls drawing. If one of them panics, only that call is lost.

`Workspaces` doesn't know which compositor is running. `src/compositor.rs` picks a backend from whichever of these is set:

- niri (`NIRI_SOCKET`)
- Hyprland (`HYPRLAND_INSTANCE_SIGNATURE`)
- Sway (`SWAYSOCK`)
- Mango (`MANGO_INSTANCE_SIGNATURE`)

Every backend hands over the full workspace list after each event. niri only sends what changed, so its backend remembers the rest and rebuilds the list itself. Hyprland and Sway are just asked for the full list again. Mango sends everything every time, and each tag counts as a workspace. `Workspaces` sorts the list by monitor and position, and if nothing changed (a window title changing is also a compositor event), it writes quietly.

To add a compositor, you add one file in `src/compositor/`, its environment variable in `running()`, and one arm in each of the two `match`es (`listen` and `focus_workspace`).

## The amane CLI

The `amane` command in `cli/` is what builds and runs your shell, and it ships the library inside itself. At compile time, `cli/build.rs` copies every file in `src/`, plus the library's `Cargo.toml` and `Cargo.lock`, into the binary with `include_bytes!`.

When you run `amane dev` or `amane compile`, `project::prepare` unpacks that copy into `~/.cache/amane/library`. It only rewrites files whose contents changed, because a newer timestamp makes cargo rebuild Amane from scratch. Then it writes a Cargo project in `~/.cache/amane/project` whose binary is your `~/.config/amane/src/main.rs`, using the library's own `Cargo.lock`, so you build against the exact versions Amane was tested with. It also writes a `Cargo.toml` next to your config, so rust-analyzer can find Amane.

The result is that the library your shell builds against is always the one that came with your `amane` binary, and you never add Amane as a dependency yourself.

`amane dev` builds, starts the shell, then watches your `src/` folder and rebuilds on every save. If a build fails, the old shell keeps running while you fix the error.

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
cli/                   the amane command: startup, dev, compile, run, clean, ipc call
examples/              one small program per feature
```

## Rules the compiler won't check

Breaking any of these gives you a crash, a freeze, or a window that silently stops updating.

Struct field order is drop order. Rust drops fields from top to bottom, and some things have to die before others. In `OpenWindow`, `gpu` comes before `fractional`, which comes before `role`, because the GPU draws into the surface and the fractional scale object belongs to that surface. In `WaylandState`, `windows` comes before `connection`. In `WaylandApp`, `state` comes before `event_loop`, because the event loop holds the connection the GPU draws through. **DO NOT reorder these fields.**

Wayland and drawing stay on the main thread. Other threads never touch `WaylandState`. They mark a change and ping (`changes::mark` or `changes::mark_all`), and the event loop does the actual work.

Widgets keep no state between frames. Anything that has to last goes in a Service or a name-keyed table.

Views read Services only through `read()`, because `read()` is what records that the window depends on it. Data you reach any other way won't redraw the window when it changes.

Never call `write()` inside a view. The view may still be holding a read of the same Service, so the write waits forever. Write from input handlers, or from the Service's own thread.

The last matching target wins. Targets are collected parents first, so hit testing has to search from the end.

Slow work stays off the main thread. Commands, file watching, image decoding, PAM and D-Bus calls all run on other threads. There's one known exception: reading the file list after a drag-and-drop blocks the loop for a moment. File managers answer instantly, so it hasn't mattered yet, but it's noted in `dropped.rs`.

## Errors, debugging and tests

If the shell can't start, like when a Wayland protocol it needs is missing, it panics with `expect("failed to <verb> <thing>")`. A shell that can't start should say why and stop. Once it's running, failures don't take it down. A D-Bus call that fails returns `Value::Nothing`, a Service whose `listen` panics restarts, a worker job that panics loses only that call, and a GPU error only loses its frame.

Set `AMANE_FRAMES=1` to print one line per frame. It shows the time since that window's last frame, how long the view, drawing and GPU steps took, and which Service woke the window.

The library tests don't need a compositor, so CI runs them on every push with `nix develop -c cargo test --lib`. The layout tests in `src/frame.rs` build a small tree and check exactly where each widget lands, so add one when you change layout. Each compositor backend has a parse test fed with trimmed replies from that compositor, so add one when you add a backend. Anything that draws still needs a real compositor, so run an example too, like `cargo run --example bar`.