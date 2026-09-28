use std::io;

use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Service, Text, Vertical};

// shows the last line typed into the terminal that started the bar
struct Typed {
    line: String,
}

impl Service for Typed {
    fn new() -> Self {
        Self {
            line: String::from("type something in the terminal"),
        }
    }

    // waiting for a line only blocks this service's thread, the bar keeps drawing
    fn listen() {
        for line in io::stdin().lines() {
            let Ok(line) = line else {
                return;
            };

            Self::write().line = line;
        }
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let typed = Typed::read();

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .child(Text::new(&typed.line).size(20.0).color(Color::WHITE)),
        )
}
