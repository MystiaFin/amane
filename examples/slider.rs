use std::cell::Cell;

use amane::{App, Color, LayerWindow, Point, Rectangle};

const TRACK_WIDTH: f32 = 300.0;
const TRACK_HEIGHT: f32 = 24.0;

thread_local! {
    // from 0 to 1, kept outside view because view is built again on every redraw
    static VALUE: Cell<f32> = const { Cell::new(0.5) };
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let filled = TRACK_WIDTH * VALUE.get();

    let fill = Rectangle::new()
        .width(filled)
        .height(TRACK_HEIGHT)
        .fill(Color::from("#3b82f6"));

    let track = Rectangle::new()
        .width(TRACK_WIDTH)
        .height(TRACK_HEIGHT)
        .radius(TRACK_HEIGHT / 2.0)
        .fill(Color::from("#e8e8e8"))
        .clip()
        .child(fill)
        .on_drag(slide);

    LayerWindow::new()
        .width(TRACK_WIDTH)
        .height(TRACK_HEIGHT)
        .child(track)
}

// the pointer can go past either end while dragging, so the value is kept in range
fn slide(point: Point) {
    let value = (point.x / TRACK_WIDTH).clamp(0.0, 1.0);

    VALUE.set(value);

    println!("value {value:.2}");
}
