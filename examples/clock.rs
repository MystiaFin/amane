use std::time::Duration;

use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Service, Text, Vertical};
use chrono::Local;

struct Clock {
    time: String,
}

impl Service for Clock {
    fn new() -> Self {
        Self { time: read_time() }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) {
        self.time = read_time();
    }
}

fn read_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let clock = Clock::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .color(Color::BLUE)
                .child(Text::new(&clock.time).size(20.0).color(Color::WHITE)),
        )
}
