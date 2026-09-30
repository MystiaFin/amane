use std::time::Duration;

use amane::{
    App, Button, Color, Full, Key, Keyboard, Layer, LayerWindow, Parent, Rectangle, Scroll,
    Service, Text, Vertical,
};

struct Counter {
    count: i32,
    hovered: bool,
}

impl Service for Counter {
    fn new() -> Self {
        Self {
            count: 0,
            hovered: false,
        }
    }

    // nothing changes on its own, only through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let counter = Counter::read();

    let fill = if counter.hovered {
        Color::rgb(90, 111, 216)
    } else {
        Color::BLUE
    };

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .keyboard(Keyboard::OnDemand)
        .on_key(key_pressed)
        .child(
            Rectangle::new()
                .width(200.0)
                .height(Parent)
                .fill(fill)
                .on_click(counter_clicked)
                .on_hover(counter_hovered)
                .on_scroll(counter_scrolled)
                .child(
                    Text::new(format!("count: {}", counter.count))
                        .size(20.0)
                        .color(Color::WHITE),
                ),
        )
}

fn counter_clicked(button: Button) {
    let mut counter = Counter::write();

    match button {
        Button::Left => counter.count += 1,
        Button::Right => counter.count -= 1,
        Button::Middle => counter.count = 0,
    }
}

fn counter_hovered(inside: bool) {
    Counter::write().hovered = inside;
}

fn counter_scrolled(scroll: Scroll) {
    let steps = scroll.y.round() as i32;

    Counter::write().count -= steps;
}

fn key_pressed(key: Key) {
    if key == Key::Escape {
        Counter::write().count = 0;
    }
}
