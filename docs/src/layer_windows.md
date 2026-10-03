# Layer Windows

A `LayerWindow` is a window that's part of the desktop rather than an app: a bar, a dock, a popup, a wallpaper. A normal app window goes wherever the compositor puts it, and other windows cover it. A layer window sticks to a screen edge, sits on a layer you choose, and can keep other windows out of its way.

This page covers each of those settings.

## A bar

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Vertical, Zone};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(32.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .child(Rectangle::new().width(Parent).height(Parent).fill(Color::BLACK))
}
```

This is a real bar: full width, 32 pixels tall, stuck to the top, above normal windows, and other windows are kept out of the 32 pixels it covers.

## How window settings reach the compositor

The compositor (niri, Hyprland, Sway) owns the screen. A `LayerWindow` is a request to it: "put this on the top edge, at this size, on this layer". Each method sets one part of that request:

- `width` and `height` come first, and are required. A number is a size in pixels. `Full` stretches the window across the whole monitor.
- `anchor_vertical` and `anchor_horizontal` pick the edges it sticks to.
- `layer` picks how high in the stack it is.
- `space` picks whether other windows have to stay out of its way.

The view runs again on every change, and Amane compares the new settings with the old ones. When one changed, Amane sends only that change to the compositor. So you can move, resize, or hide a window just by returning different settings from the view.

## Size

```rust,ignore
LayerWindow::new().width(300.0).height(200.0)   // 300 by 200 pixels
LayerWindow::new().width(Full).height(30.0)     // full width, 30 pixels tall
LayerWindow::new().width(48.0).height(Full)     // full height, for a side bar
```

`width` must come before `height`, and both must come before anything else. If you forget one, the code doesn't compile, so you can't open a window without a size.

A compositor may give a window a different size than it asked for. Use `window_size()` inside a view to read the real one ([Multiple Windows and Monitors](windows.md)).

## Position

```rust,ignore
.anchor_vertical(Vertical::Top)          // Top, Middle, or Bottom
.anchor_horizontal(Horizontal::Right)    // Left, Middle, or Right
```

Both default to `Middle`, so a window with no anchors sits in the center of the screen.

A `Full` size anchors both opposite edges on its own. A `Full` width is stuck to the left and the right edge, whatever `anchor_horizontal` says.

To keep a gap from the edges, use a margin. The numbers are pixels:

```rust,ignore
.margin(Margin { top: 10, right: 10, bottom: 0, left: 0 })
```

A margin only pushes away from an edge the window is anchored to. A negative margin pushes the window past the edge, partly off screen, which is useful for sliding a panel in and out ([Animation](animation.md)).

## Layer

```rust,ignore
.layer(Layer::Top)
```

| Layer | Sits | Use it for |
|---|---|---|
| `Background` | under everything | wallpapers |
| `Bottom` | under normal windows | desktop widgets |
| `Top` | above normal windows | bars, docks |
| `Overlay` | above everything, even fullscreen windows | popups, launchers, OSDs |

The default is `Overlay`. Set `Top` for a bar, or it covers fullscreen videos.

## Reserved space

```rust,ignore
.space(Zone::Reserve)
```

| Zone | Effect |
|---|---|
| `Reserve` | Other windows are kept out of the strip this window covers. Use it for bars. |
| `Respect` | Reserves nothing, and moves out of the space other windows reserve. This is the default. |
| `Ignore` | Reserves nothing, and covers the space other windows reserve too. Use it for full-screen overlays. |

`Reserve` reserves the window's height, or its width for a window anchored to the left or right edge with a fixed width.

## Keyboard

A layer window gets no keys by default. To type into it, ask for keyboard focus:

```rust,ignore
.keyboard(Keyboard::OnDemand)
```

| Keyboard | Effect |
|---|---|
| `None` | Never gets keys. This is the default. |
| `OnDemand` | Gets keys after you click it, like a normal window. Use it for a search box in a panel. |
| `Exclusive` | Takes all keys as long as it's open. Use it for launchers and menus. |

[Input](input.md) covers what to do with the keys.

## Hiding and showing

```rust,ignore
.visible(panel.open)
```

A hidden window stays alive. Its view keeps running, and it shows again when the view returns `visible(true)`. This is how you make a panel you toggle with a keybind ([IPC](ipc.md)).

## Clicks that pass through

By default, the whole window takes the mouse. To let clicks reach the windows underneath:

```rust,ignore
.click_through()                     // no part of the window takes the mouse
.input_region(vec![InputArea { x: 0, y: 0, width: 100, height: 30 }])   // only this part does
```

`InputArea` is in pixels, measured from the window's top-left corner. You can pass several.

## Namespace

```rust,ignore
.namespace("bar")
```

The namespace is a name the compositor can match window rules on, like niri's `layer-rule`. It defaults to `"amane"`. It's only read when the window opens, so changing it later does nothing.

## Terms

- **Layer shell:** the Wayland protocol (wlr-layer-shell) that layer windows use. Your compositor must support it.
- **Anchor:** an edge a window is stuck to.
- **Exclusive zone:** the compositor's name for reserved space.
