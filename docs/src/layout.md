# Layout

A bar usually has a few things on the left, a clock in the middle, and a few things on the right. When the bar's width changes, the middle should stretch, and the sides should stay the same size.

This page covers how widgets get their size and position: `Size`, `Row`, `Column`, `Stack`, and the spacing and alignment options.

## Fixed and stretching widgets

```rust
use amane::{App, Color, Full, LayerWindow, Parent, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Row::new(children![
            Rectangle::new().width(100.0).height(Parent).fill(Color::RED),
            Rectangle::new().width(Parent).height(Parent).fill(Color::GREEN),
            Rectangle::new().width(100.0).height(Parent).fill(Color::BLUE),
        ]))
}
```

Red is always 100 pixels, blue is always 100 pixels, and green takes whatever is left.

## How a row places its children

Every widget answers one question for each direction: how big do you want to be? The answer is a `Size`, and there are two kinds:

- **a number**, like `100.0`: exactly that many pixels.
- **`Parent`**: "as big as my parent lets me".

The `Row` works out positions in two steps:

1. **It measures.** The fixed children need 100 + 100 = 200 pixels. The rest of the row's width is free.
2. **It places.** Each child is put next to the one before it. The free space is split evenly between the `Parent` children. Here there's only one, so green gets all of it.

When the bar gets wider, only the free space changes, so only green grows.

## Lists of widgets

`Row::new` takes a list of widgets of different types: rectangles, text, other rows. Rust lists can only hold one type, so each widget has to be put in a `Box<dyn Widget>`, which means "any widget". The `children!` macro does that boxing for you:

```rust,ignore
Row::new(children![
    Rectangle::new().width(100.0).height(30.0),
    Text::new("hello"),
])
```

When you build the list in a loop, box each widget yourself:

```rust
use amane::{App, Column, LayerWindow, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for number in 1..=5 {
        rows.push(Box::new(Text::new(format!("row {number}"))));
    }

    LayerWindow::new().width(200.0).height(200.0).child(Column::new(rows))
}
```

## Row, Column, and Stack

| Widget | Places children |
|---|---|
| `Row` | side by side, left to right |
| `Column` | on top of each other, top to bottom |
| `Stack` | all in the same spot, later children drawn over earlier ones |

`Row` and `Column` work the same way, just in different directions. Everything on this page about `Row` also applies to `Column`, with width and height swapped.

`Stack` is for layering, like a badge on top of a card. Each child starts at the stack's top-left corner. Move one with `translate` ([Rectangle](rectangle.md)):

```rust
use amane::{App, Center, Color, Full, LayerWindow, Rectangle, Stack, Text, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let card = Rectangle::new()
        .width(200.0)
        .height(120.0)
        .radius(16.0)
        .fill("#1e1e2e");

    let badge = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(Full)
        .fill("#f38ba8")
        .translate(186.0, -14.0)
        .align_child(Center, Center)
        .child(Text::new("3").size(14.0).color(Color::BLACK));

    LayerWindow::new()
        .width(260.0)
        .height(180.0)
        .child(Stack::new(children![card, badge]))
}
```

## How big a row is

If you don't give a `Row` a size, it measures its children:

- Its width is all its children's widths added up, plus the gaps.
- Its height is its tallest child's height.
- If any child is `Parent`-sized in a direction, the row becomes `Parent`-sized in that direction too.

To set a size yourself, use `.width(...)` and `.height(...)`. Both are optional on `Row`, `Column`, and `Stack`.

## Gaps

```rust,ignore
Row::new(children![a, b, c]).gap(12.0)
```

`gap` puts 12 pixels between each pair of children. There's no gap before the first child or after the last.

## Spreading children along a row

When the children don't fill the row, `justify` decides where the leftover space goes:

```rust,ignore
Row::new(children![a, b, c]).width(Parent).justify(SpaceBetween)
```

| Justify | Effect |
|---|---|
| `Start` | packed at the start. This is the default. |
| `Center` | packed in the middle |
| `End` | packed at the end |
| `SpaceBetween` | first at the start, last at the end, equal space between |
| `SpaceAround` | equal space around each child |
| `SpaceEvenly` | equal space between every child and the edges |

`justify` only matters when there's space left over. A `Parent`-sized child takes all of it, so there's nothing to spread.

## Lining children up across a row

`align` decides where each child sits in the other direction. For a `Row`, that's vertically:

```rust,ignore
Row::new(children![a, b, c]).height(40.0).align(Center)
```

It takes `Start`, `Center`, or `End`. The default is `Start`: the top of a row, or the left of a column.

## Padding and placing a single child

A `Rectangle` holds one child. `padding` keeps space between its edges and the child, and `align_child` places the child inside that space:

```rust
use amane::{App, Center, Color, End, LayerWindow, Padding, Parent, Rectangle};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(300.0).height(100.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Color::BLACK)
            .padding(Padding { top: 8.0, right: 16.0, bottom: 8.0, left: 16.0 })
            .align_child(End, Center)
            .child(Rectangle::new().width(40.0).height(40.0).fill(Color::RED)),
    )
}
```

- `padding(16.0)` puts the same space on every side. `Padding { ... }` sets each side.
- `align_child(horizontal, vertical)` takes `Start`, `Center`, or `End` for each direction. The default is the top-left corner.

## Example: a three-part bar

Left, center, and right sections, with the center always in the middle of the bar:

```rust
use amane::{
    Align, App, Center, Color, End, Full, LayerWindow, Parent, Rectangle, Row, Start, Text,
    children,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .child(Row::new(children![
            section(Text::new("workspaces").color(Color::WHITE), Start),
            section(Text::new("12:00").color(Color::WHITE), Center),
            section(Text::new("battery").color(Color::WHITE), End),
        ]))
}

fn section(content: Text, align: impl Into<Align>) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill("#1e1e2e")
        .align_child(align, Center)
        .child(content)
}
```

All three sections are `Parent`-sized, so each gets exactly a third of the bar. The middle third is always centered, however long the left and right content is.

## Terms

- **Size:** a fixed number of pixels, or `Parent`.
- **Justify:** placement along the direction a row or column grows.
- **Align:** placement across that direction.
- **Main axis / cross axis:** the usual names for "along" and "across".
