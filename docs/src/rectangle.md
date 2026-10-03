# Rectangle

`Rectangle` is the widget you'll use most. It's a box that can be painted, rounded, outlined, shadowed, and transformed, and it can hold one child. Buttons, cards, backgrounds, and icons are all rectangles.

## A rounded rectangle

```rust
use amane::{App, LayerWindow, Rectangle};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(200.0)
        .height(100.0)
        .child(Rectangle::new().width(160.0).height(60.0).radius(12.0).fill("#89b4fa"))
}
```

`width` and `height` are required, and must come first, in that order. They take a number of pixels or `Parent` ([Layout](layout.md)). Everything else is optional.

## Fill

`fill` takes any of these:

| Fill | Example |
|---|---|
| a hex string | `.fill("#1e1e2e")`, also `"#rgb"` and `"#rrggbbaa"` |
| a `Color` | `.fill(Color::BLUE)`, `.fill(Color::rgb(30, 30, 46))`, `.fill(Color::rgba(0, 0, 0, 128))` |
| a `Gradient` | `.fill(Gradient::linear(90.0, [(0.0, "#89b4fa"), (1.0, "#f5c2e7")]))` |
| an `Image` | `.fill(Image::cover("/path/to/wallpaper.png"))`, see [Images](images.md) |
| `Mask` | `.fill(Mask)`, a hole, see [Masks](#masks) |

A rectangle with no fill is invisible, which is useful for spacing and for invisible click areas.

A mistyped hex string, like `"#12345"`, shows bright pink instead of crashing your shell, so you can spot it.

`Color` has the constants `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, and `TRANSPARENT`. `Color::rgb` and `Color::rgba` are `const`, so you can keep your theme as constants:

```rust,ignore
const BACKGROUND: Color = Color::rgb(0x1e, 0x1e, 0x2e);
const FOREGROUND: Color = Color::rgb(0xcd, 0xd6, 0xf4);
```

## Gradients

```rust,ignore
Gradient::linear(90.0, [(0.0, "#89b4fa"), (1.0, "#f5c2e7")])
Gradient::radial([(0.0, "#f9e2af"), (0.5, "#fab387"), (1.0, "#1e1e2e")])
```

Each stop is a position from 0 to 1 and a color. A linear gradient also takes an angle in degrees, the same way CSS does: `0.0` goes from bottom to top, `90.0` from left to right, and `180.0` from top to bottom. A radial gradient goes from the center out to the edges.

## Corners and borders

```rust,ignore
.radius(12.0)            // rounded corners, 12 pixels
.radius(Full)            // a pill, or a circle if the rectangle is square
.border(2.0, "#cdd6f4")  // a 2 pixel outline along the inside edge
```

`Full` makes the radius half of the shorter side, so it stays a perfect pill however the rectangle is sized.

## Opacity, shadow, and blur

```rust,ignore
.opacity(0.5)
.shadow(Shadow::drop("#000000").opacity(0.3).blur(12.0).offset(0.0, 4.0))
.shadow(Shadow::inner("#000000").opacity(0.3).blur(8.0))
.blur(20.0)
```

- `opacity` fades the rectangle **and everything inside it**, from 0 (invisible) to 1.
- `Shadow::drop` casts a shadow behind the rectangle. `Shadow::inner` shades along the inside edge, as if it were pressed in. `blur` softens it, and `offset` moves it.
- `blur` blurs whatever was drawn behind the rectangle, inside its shape, for a frosted-glass look. It can only blur what your own window drew underneath. It can't see other apps' windows.

## Masks

A rectangle filled with `Mask` paints nothing. Instead, it cuts its shape out of the nearest rectangle around it:

```rust
use amane::{App, Center, Full, LayerWindow, Mask, Parent, Rectangle};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(200.0).height(120.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .radius(24.0)
            .fill("#1e1e2e")
            .align_child(Center, Center)
            .child(Rectangle::new().width(60.0).height(60.0).radius(Full).fill(Mask)),
    )
}
```

The window gets a round hole in it, and you see your wallpaper through it. A mask can have a `border`, to outline the hole.

## Holding a child

```rust,ignore
Rectangle::new()
    .width(Parent)
    .height(Parent)
    .padding(8.0)
    .align_child(Center, Center)
    .clip()
    .child(Text::new("hello"))
```

- `child` puts one widget inside. To hold several, give it a `Row`, `Column`, or `Stack`.
- `padding` and `align_child` place the child ([Layout](layout.md)).
- `clip` hides any part of the child that sticks out, following the rounded corners. Use it for scrolling lists and progress bars.

## Transforms

```rust,ignore
.rotate(30.0)          // degrees, clockwise
.scale(0.6)            // 1.0 keeps the size
.translate(20.0, -30.0)
```

Rotation and scale happen around the rectangle's center. Transforms move the drawing and the click area, but not the layout: the rectangle still takes up its original space in its row or column, and its neighbors don't move. That makes them good for animation and for overlapping things in a `Stack`.

The child moves with its rectangle.

## Input

`on_click`, `on_hover`, `on_scroll`, `on_move`, `on_drag`, and `cursor` are covered in [Input](input.md).

## Shaders

`.shader("path/to/file.wgsl")` draws a GPU shader over the fill. See [Shaders](shaders.md).

## Example: a button

Putting it together:

```rust
use amane::{App, Center, Color, LayerWindow, Pointer, Rectangle, Shadow, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(200.0).height(80.0).child(button("Save"))
}

fn button(label: &str) -> Rectangle {
    Rectangle::new()
        .width(120.0)
        .height(36.0)
        .radius(8.0)
        .fill("#89b4fa")
        .shadow(Shadow::drop("#000000").opacity(0.25).blur(8.0).offset(0.0, 2.0))
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| println!("saved"))
        .child(Text::new(label).color(Color::from("#1e1e2e")))
}
```
