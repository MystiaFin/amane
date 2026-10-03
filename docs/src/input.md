# Input

A bar needs buttons: click a workspace to switch to it, scroll over the volume to change it, click the clock to open a calendar.

This page covers mouse and keyboard input: clicks, hover, scrolling, dragging, the pointer's look, and keys.

## Handling a click

```rust
use amane::{App, Button, LayerWindow, Parent, Rectangle, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(200.0).height(40.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill("#1e1e2e")
            .on_click(clicked)
            .child(Text::new("click me")),
    )
}

fn clicked(button: Button) {
    println!("clicked with {button:?}");
}
```

## How input reaches a handler

Input handlers are set on a `Rectangle`. Each frame, Amane remembers where every rectangle with a handler ended up. When you click, it finds the rectangle under the pointer and calls its handler.

If rectangles overlap, the one drawn last gets the event. That's the innermost and topmost one: a button inside a panel gets the click, not the panel.

The handler runs after the frame is gone, so it can't change widgets directly. It changes state instead, usually by writing to a Service, and the view shows the change on the next frame:

```rust,ignore
fn clicked(_: Button) {
    Counter::write().count += 1;
}
```

## Mouse handlers

All of these are methods on `Rectangle`:

| Method | Called | Gets |
|---|---|---|
| `on_click` | when a button is pressed and released over the same rectangle | which `Button`: `Left`, `Right`, or `Middle` |
| `on_hover` | when the pointer comes in, and when it goes out | `true` when it comes in, `false` when it goes out |
| `on_scroll` | for each wheel step or touchpad movement | a `Scroll` with `x` and `y` |
| `on_move` | on every movement while the pointer is over it | a `Point` |
| `on_drag` | on a left press, and on every movement until release | a `Point` |

A `Point` has `x` and `y`, measured in pixels from the rectangle's top-left corner. During `on_drag`, the point keeps coming even after the pointer leaves the rectangle, so it can be negative or bigger than the rectangle. Clamp it.

A `Scroll` is measured in lines, so a mouse wheel and a touchpad move things by the same amount. Positive `y` means scrolling down, and positive `x` means scrolling right.

## Passing values into handlers

A handler can be a function, like `clicked` above, or a closure. A closure can carry values from the view into the handler:

```rust
use amane::{App, Column, LayerWindow, Parent, Rectangle, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for number in 1..=3 {
        let row = Rectangle::new()
            .width(Parent)
            .height(30.0)
            .on_click(move |_| println!("clicked row {number}"))
            .child(Text::new(format!("row {number}")));

        rows.push(Box::new(row));
    }

    LayerWindow::new().width(200.0).height(90.0).child(Column::new(rows))
}
```

`move` copies `number` into the closure, so each row remembers its own number. `|_|` ignores the `Button` argument.

A closure can't borrow from a Service's `read()`, because the handler lives longer than the view. Copy out what you need first:

```rust,ignore
let id = workspace.id();                                   // a copy, not a borrow
let name = String::from(device.name());                    // an owned copy of the text

Rectangle::new()
    .width(30.0)
    .height(Parent)
    .on_click(move |_| Workspaces::focus(id))
```

## Hover effects

A hover effect needs state, because the view has to know whether the pointer is inside:

```rust
use std::time::Duration;

use amane::{App, Color, LayerWindow, Parent, Rectangle, Service};

struct Hover {
    inside: bool,
}

impl Service for Hover {
    fn new() -> Self {
        Self { inside: false }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let hover = Hover::read();

    let fill = if hover.inside { Color::from("#45475a") } else { Color::from("#1e1e2e") };

    LayerWindow::new().width(200.0).height(40.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(fill)
            .on_hover(|inside| Hover::write().inside = inside),
    )
}
```

## Changing the pointer

```rust,ignore
Rectangle::new()
    .width(100.0)
    .height(30.0)
    .cursor(Pointer)
    .on_click(clicked)
```

While the pointer is over the rectangle, it takes that look. The names are values you import from `amane`: `Default`, `Pointer` (a hand), `Text`, `Grab`, `Grabbing`, `Move`, `NotAllowed`, `Wait`, `Crosshair`, and the resize arrows `ResizeTop`, `ResizeBottom`, `ResizeLeft`, `ResizeRight`, `ResizeTopLeft`, `ResizeTopRight`, `ResizeBottomLeft`, `ResizeBottomRight`, `ResizeHorizontal`, and `ResizeVertical`.

`Text` the cursor and `Text` the widget share a name, and both come from `amane`. Rust tells them apart by how they're used. If you'd rather be explicit, write `Cursor::Text`.

A `TextInput` shows the text cursor on its own.

## Transformed rectangles

Input follows a rectangle's transform. A rotated button only reacts inside its rotated shape, not inside the box it took up before rotating ([Rectangle](rectangle.md)).

## Keyboard input

Keys go to a window, not a widget. First the window has to ask for keyboard focus ([Layer Windows](layer_windows.md)), then it handles keys with `on_key`:

```rust
use std::process;

use amane::{App, Key, Keyboard, LayerWindow, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(300.0)
        .height(40.0)
        .keyboard(Keyboard::Exclusive)
        .on_key(key_pressed)
        .child(Text::new("press escape to quit"))
}

fn key_pressed(key: Key) {
    if key == Key::Escape {
        process::exit(0);
    }
}
```

`std::process` is Rust's standard library.

`Key` is one of:

- `Character(char)`: a key that types something, already shifted, like `Character('A')`
- `Enter`, `Escape`, `Tab`, `Backspace`, `Space`
- `Up`, `Down`, `Left`, `Right`, `Home`, `End`
- `Other`: any key Amane has no name for yet

### When a text input has focus

A focused `TextInput` takes keys before the window does, because typing should type:

- Letters, `Space`, `Backspace`, `Enter`, `Left`, `Right`, `Home`, and `End` go to the input only.
- `Up`, `Down`, `Tab`, and `Other` skip the input and go to the window's `on_key`. That's how arrow keys can move through a list while you type in a search box.
- `Escape` takes focus away from the input, and then also goes to the window, which may want to close.

## Terms

- **Handler:** a function Amane calls when an event happens.
- **Target:** the area a rectangle with handlers took up in the last frame.
- **Keyboard focus:** the window the compositor sends keys to.
