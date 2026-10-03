# Scroll Area

`ScrollArea` shows part of a widget that's too tall for its space, and scrolls it with the mouse wheel or touchpad.

## A scrolling list

```rust
use amane::{App, Column, LayerWindow, Rectangle, ScrollArea, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for number in 1..=50 {
        rows.push(Box::new(Text::new(format!("item {number}"))));
    }

    LayerWindow::new().width(300.0).height(400.0).child(
        Rectangle::new()
            .width(300.0)
            .height(400.0)
            .fill("#ffffff")
            .child(ScrollArea::new("items", Column::new(rows))),
    )
}
```

## How scrolling works

- A `ScrollArea` is `Parent`-sized by default, so it takes all the room its parent gives it. Here, that's 300 by 400 pixels.
- Its child, the column, is as tall as its 50 rows. Only the part that fits inside the area is shown.
- The view is rebuilt on every frame, so the scroll area can't remember how far it's scrolled. Amane keeps the position for you under the name you pass, here `"items"`. Give each scroll area its own name. Two areas with the same name scroll together.

## Options

```rust,ignore
ScrollArea::new("apps", Column::new(rows))
    .width(300.0)
    .height(Parent)
```

`width` and `height` default to `Parent`.

Scrolling is vertical only.

## Rounded corners

Put the scroll area inside a rounded rectangle with `clip`, so rows scrolling past the corners are cut off along the curve:

```rust,ignore
Rectangle::new()
    .width(300.0)
    .height(400.0)
    .radius(24.0)
    .fill(Color::WHITE)
    .clip()
    .child(ScrollArea::new("items", Column::new(rows)))
```

## Rows that take input

Rows inside a scroll area take clicks like anywhere else. Rows scrolled out of view can't be clicked.

A row with its own `on_scroll`, like a volume slider, takes the scroll while the pointer is over it, and the area doesn't move.
