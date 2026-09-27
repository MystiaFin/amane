use std::sync::{LazyLock, Mutex};

use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

static MESSAGE: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("run: amane ipc call say hello")));

fn main() {
    App::new().ipc("say", say).window(view).run();
}

// `amane ipc call say <words...>` shows the words in the bar
fn say(arguments: &[String]) -> String {
    let words = arguments.join(" ");

    let mut message = MESSAGE.lock().expect("failed to lock message");

    *message = words.clone();

    format!("showing: {words}")
}

fn view() -> LayerWindow {
    let message = MESSAGE.lock().expect("failed to lock message");

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
                .child(Text::new(message.as_str()).size(20.0).color(Color::WHITE)),
        )
}
