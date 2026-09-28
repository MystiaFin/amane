use amane::{
    App, Button, Color, Full, Layer, LayerWindow, Media, Parent, Rectangle, Row, Service, Text,
    Vertical, children,
};

// play something in a player like mpv or firefox while the bar is running
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let media = Media::read();

    let symbol = if media.playing() { "pause" } else { "play" };

    let seconds = media.position().as_secs();
    let total = media.length().as_secs();

    let label = format!(
        "{} - {}  {}:{:02} / {}:{:02}",
        media.artist(),
        media.title(),
        seconds / 60,
        seconds % 60,
        total / 60,
        total % 60,
    );

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Row::new(children![
            button("prev", previous_clicked),
            button(symbol, play_pause_clicked),
            button("next", next_clicked),
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .child(Text::new(label).size(20.0).color(Color::WHITE)),
        ]))
}

fn button(label: &str, clicked: fn(Button)) -> Rectangle {
    Rectangle::new()
        .width(80.0)
        .height(Parent)
        .fill(Color::BLUE)
        .on_click(clicked)
        .child(Text::new(label).size(20.0).color(Color::WHITE))
}

fn previous_clicked(_: Button) {
    Media::previous();
}

fn play_pause_clicked(_: Button) {
    Media::play_pause();
}

fn next_clicked(_: Button) {
    Media::next();
}
