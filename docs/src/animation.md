# Animation

When a panel jumps from 100 pixels tall to 400 in one frame, it looks broken. It should grow over a fraction of a second. But a view only describes one frame, and it only runs when something changes. Animations solve both.

This page covers smooth motion: sliding panels, fading popups, and colors that blend into each other.

## Growing a rectangle

```rust
use std::time::Duration;

use amane::{Animation, App, Button, LayerWindow, Parent, Rectangle, Service};

struct Panel {
    height: Animation,
}

impl Service for Panel {
    fn new() -> Self {
        Self { height: Animation::new(100.0) }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let panel = Panel::read();

    LayerWindow::new().width(300.0).height(400.0).child(
        Rectangle::new()
            .width(Parent)
            .height(panel.height.value())
            .fill("#89b4fa")
            .on_click(clicked),
    )
}

fn clicked(_: Button) {
    let mut panel = Panel::write();

    let target = if panel.height.value() < 250.0 { 400.0 } else { 100.0 };

    panel.height.to(target);
}
```

Click the rectangle and it grows smoothly to 400 pixels. Click again and it shrinks back.

## How an animation keeps drawing

An `Animation` doesn't store a changing number. It stores where it started, where it's going, and when it started. `value()` works out the current number from the clock.

1. `to(400.0)` sets a new target and notes the time. Because this happens inside `write()`, the window redraws.
2. The view calls `value()`. The animation has just started, so it returns about 100.
3. Calling `value()` on an unfinished animation also tells Amane "I'm still moving". After drawing, Amane asks the compositor for the next frame.
4. On the next frame, the view runs again, and `value()` returns a little more.
5. Once the animation arrives, `value()` stops asking for frames, and the window rests.

So an animation only costs frames while it's moving. A shell full of finished animations draws nothing.

## Speed and easing

```rust,ignore
Animation::new(0.0)
    .duration(Duration::from_millis(400))
    .easing(Easing::InOut)
```

- `duration` defaults to 200 milliseconds.
- `easing` decides how the speed changes along the way:

| Easing | Motion |
|---|---|
| `Out` | starts fast, slows down as it arrives. This is the default, and feels right for most UI. |
| `InOut` | starts slow, speeds up, slows down again. Good for big movements. |
| `Linear` | the same speed the whole way |

## Changing direction halfway

If you call `to` while an animation is still moving, it starts again from wherever it is right now. It never jumps. So you can click a toggle quickly several times, and the panel just turns around smoothly.

Calling `to` with the target it already has does nothing.

## Animating colors

`Animation` works with `f32` (the default) and with `Color`:

```rust,ignore
struct Tab {
    fill: Animation<Color>,
}

// in new()
fill: Animation::new(Color::from("#1e1e2e")),

// in a hover handler
Tab::write().fill.to(Color::from("#45475a"));

// in the view
.fill(tab.fill.value())
```

To animate your own type, implement the `Blend` trait for it. `blend(from, to, amount)` returns the value `amount` of the way from `from` to `to`, where `amount` goes from 0 to 1.

## Animating windows

Anything the view returns can come from an animation, including the window's own size and margin. A panel that slides in from the top edge:

```rust
use std::time::Duration;

use amane::{
    Animation, App, Horizontal, Layer, LayerWindow, Margin, Parent, Rectangle, Service, Vertical,
};

struct Panel {
    open: bool,
    slide: Animation,
}

impl Service for Panel {
    fn new() -> Self {
        Self { open: true, slide: Animation::new(10.0).duration(Duration::from_millis(300)) }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().ipc("toggle", toggle).window(view).run();
}

fn view() -> LayerWindow {
    let panel = Panel::read();

    let margin = Margin { top: panel.slide.value() as i32, right: 10, bottom: 0, left: 0 };

    LayerWindow::new()
        .width(300.0)
        .height(100.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Top)
        .margin(margin)
        .child(Rectangle::new().width(Parent).height(Parent).fill("#89b4fa"))
}

fn toggle(_: &[String]) -> String {
    let mut panel = Panel::write();

    panel.open = !panel.open;

    let target = if panel.open { 10.0 } else { -110.0 };

    panel.slide.to(target);

    String::from("ok")
}
```

`amane ipc call toggle` slides the panel up past the screen edge (a negative margin), and back down the next time ([IPC](ipc.md)).

## Motion you control yourself

Some motion never arrives, like a spinner or a pulsing dot. For that, compute the position from the time yourself, and call `request_frame()` to ask for another frame:

```rust
use std::sync::LazyLock;
use std::time::Instant;

use amane::{App, Full, LayerWindow, Rectangle};

static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let seconds = START.elapsed().as_secs_f32();

    let x = 170.0 + (seconds * 3.0).sin() * 150.0;

    // it never arrives, so it always wants the next frame
    amane::request_frame();

    LayerWindow::new().width(400.0).height(60.0).child(
        Rectangle::new()
            .width(60.0)
            .height(60.0)
            .radius(Full)
            .fill("#89b4fa")
            .translate(x, 0.0),
    )
}
```

`LazyLock` and `Instant` come from Rust's standard library.

Stop calling `request_frame()` once the motion is done, so the window can rest. A view that always calls it redraws at the monitor's refresh rate forever.

## Terms

- **Easing:** the curve that maps time passed to distance traveled.
- **Frame callback:** the compositor's signal that it's ready for the next frame. Amane waits for it, so animations run at the monitor's refresh rate.
