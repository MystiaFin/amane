use std::fs;

use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Service, Text, Vertical};

// try `echo hello > /tmp/amane-note` while the bar is running
const NOTE: &str = "/tmp/amane-note";

struct Note {
    text: String,
}

impl Service for Note {
    fn new() -> Self {
        Self { text: read_note() }
    }

    fn listen() {
        for _ in amane::watch_file(NOTE) {
            let text = read_note();

            Self::write().text = text;
        }
    }
}

fn read_note() -> String {
    let Ok(text) = fs::read_to_string(NOTE) else {
        return format!("no note yet, write one to {NOTE}");
    };

    String::from(text.trim_end())
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let note = Note::read();

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
                .child(Text::new(&note.text).size(20.0).color(Color::WHITE)),
        )
}
