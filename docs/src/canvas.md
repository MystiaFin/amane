# Canvas and Shapes

`Canvas` draws shapes that rectangles can't: progress rings, gauges, graphs, and custom icons.

## A progress ring

A progress ring at 70%:

```rust
use amane::{App, Arc, Canvas, Cap, LayerWindow, Shape, shapes};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(120.0).height(120.0).child(ring(0.7))
}

fn ring(value: f32) -> Canvas {
    Canvas::new().width(120.0).height(120.0).shapes(shapes![
        Arc::new().stroke(10.0, "#45475a"),
        Arc::new().sweep(360.0 * value).stroke(10.0, "#89b4fa").cap(Cap::Round),
    ])
}
```

## Coordinates and drawing order

- A `Canvas` is a widget with a size, like a rectangle. Inside it, shapes are placed in the canvas's own coordinates: `(0, 0)` is its top-left corner, and `x` and `y` grow right and down.
- `shapes!` makes the list of shapes, like `children!` does for widgets. Later shapes are drawn over earlier ones, so the blue arc sits on top of the gray track.
- An `Arc` with no center and no radius sits in the middle of the canvas, as big as fits, and pulls in by half its line width so the line stays inside.
- `stroke`, `fill`, `cap`, and `opacity` come from the `Shape` trait, so `Shape` has to be in your `use` line, or Rust can't find these methods.

## Shapes

| Shape | Builder | Defaults |
|---|---|---|
| `Circle` | `.center(x, y)`, `.radius(r)` | middle of the canvas, as big as fits |
| `Arc` | `.center(x, y)`, `.radius(r)`, `.start(degrees)`, `.sweep(degrees)` | middle of the canvas, as big as fits, a full turn |
| `Line` | `.from(x, y)`, `.to(x, y)` | from `(0, 0)` to `(0, 0)` |
| `Path` | `.move_to`, `.line_to`, `.quad_to`, `.cubic_to`, `.arc`, `.close` | empty |

Arc angles are in degrees. `0` points straight up, and angles grow clockwise. A negative sweep goes counterclockwise. `Path::arc` uses the same angles.

## Fill, stroke, and caps

Every shape gets the same four methods:

```rust,ignore
.fill("#89b4fa")          // paints the inside
.stroke(4.0, "#cdd6f4")   // draws the outline, 4 pixels thick
.cap(Cap::Round)          // how open line ends look: Butt (the default), Round, or Square
.opacity(0.5)
```

A new shape draws nothing until it gets a fill or a stroke. It can have both.

## Paths and graphs

A `Path` is a pen you move around. Each call adds to the same path:

```rust,ignore
Path::new()
    .move_to(10.0, 50.0)                      // lift the pen and put it here
    .line_to(50.0, 10.0)                      // a straight line
    .quad_to(70.0, 0.0, 90.0, 10.0)           // a curve with one handle
    .cubic_to(100.0, 30.0, 100.0, 70.0, 90.0, 90.0)  // a curve with two handles
    .close()                                  // a straight line back to the last move_to
    .stroke(2.0, "#cdd6f4")
```

Paths are good for graphs. Build them in a loop:

```rust
use amane::{App, Canvas, Cap, LayerWindow, Path, Shape, shapes};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let history = [0.2, 0.5, 0.4, 0.8, 0.6, 0.9, 0.3];

    LayerWindow::new().width(200.0).height(60.0).child(graph(&history))
}

fn graph(history: &[f32]) -> Canvas {
    let width = 200.0;
    let height = 60.0;

    let step = width / (history.len() - 1) as f32;

    let mut line = Path::new().move_to(0.0, height * (1.0 - history[0]));

    for (index, value) in history.iter().enumerate().skip(1) {
        line = line.line_to(index as f32 * step, height * (1.0 - value));
    }

    Canvas::new()
        .width(width)
        .height(height)
        .shapes(shapes![line.stroke(2.0, "#a6e3a1").cap(Cap::Round)])
}
```

`y` grows downward, so a value of 1 is drawn at `y = 0`, the top.

## Example: a gauge

Half a circle, with a needle:

```rust
use amane::{App, Arc, Canvas, Cap, Circle, LayerWindow, Line, Shape, shapes};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(120.0).height(80.0).child(gauge(0.4))
}

fn gauge(value: f32) -> Canvas {
    let angle = f32::to_radians(-90.0 + 180.0 * value);

    let needle_x = 60.0 + f32::sin(angle) * 36.0;
    let needle_y = 70.0 - f32::cos(angle) * 36.0;

    Canvas::new().width(120.0).height(80.0).shapes(shapes![
        Arc::new()
            .center(60.0, 70.0)
            .radius(48.0)
            .start(-90.0)
            .sweep(180.0)
            .stroke(8.0, "#45475a")
            .cap(Cap::Round),
        Arc::new()
            .center(60.0, 70.0)
            .radius(48.0)
            .start(-90.0)
            .sweep(180.0 * value)
            .stroke(8.0, "#f9e2af")
            .cap(Cap::Round),
        Line::new().from(60.0, 70.0).to(needle_x, needle_y).stroke(3.0, "#cdd6f4").cap(Cap::Round),
        Circle::new().center(60.0, 70.0).radius(6.0).fill("#cdd6f4"),
    ])
}
```

## When to use a canvas

- For boxes, pills, circles, and anything with a shadow, use `Rectangle`. It's simpler, and it can hold children and take input.
- For arcs, lines, curves, and graphs, use `Canvas`.

A canvas can't hold children or take input on its own. To make one clickable, put it inside a rectangle that has the handler.
