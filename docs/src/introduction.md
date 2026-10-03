# Introduction

Amane is a Rust library for building your own Wayland desktop shell: bars, panels, launchers, notification popups, and lock screens. You write the whole shell as one Rust program. It connects straight to the compositor and draws on the GPU, with no browser engine, no QML, and no GTK underneath.

Here's a complete bar:

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
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
                .child(Text::new("Amane bar").size(20.0).color(Color::WHITE)),
        )
}
```

That's a 30-pixel blue strip across the top of the screen with some white text in it.

## What you need

- A Wayland compositor that supports **wlr-layer-shell**, like niri, Hyprland, Sway, or river. GNOME doesn't support it.
- A GPU with Vulkan drivers.
- Some Rust. You don't need to know it deeply. If you can read a function, a struct, and a closure (`|x| ...`), you can follow this book. Where Amane uses something less common, the page explains it.

## How this book is laid out

- **Getting Started** installs Amane and gets a bar on your screen.
- **Core Concepts** explains the ideas everything else is built on: views, windows, layout, state, input, and animation. Read these in order.
- **Widgets** covers each building block in detail.
- **More Windows** covers multiple monitors, normal windows, and the lock screen.
- **Built-in Services** covers the system data Amane can read for you: battery, audio, network, media players, notifications, and more.
- **Talking to the System** covers running commands, watching files, IPC, and D-Bus.

The Core Concepts pages build on each other, and each one starts from a problem a shell has and works up to how Amane solves it. The other pages are for looking things up: each starts with a short example, followed by sections named after what you might want to do.

Every code block starts with its `use` line, so you always know where a name comes from. Unless a page says otherwise, every name comes from `amane`.

If you want to know how Amane works inside, read `ARCHITECTURE.md` in the repository instead.
