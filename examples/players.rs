use amane::{
    App, Color, Column, Full, Layer, LayerWindow, Media, MediaPlayer, Parent, Rectangle, Row,
    Service, Text, Vertical, Widget, children,
};

const ROW_HEIGHT: f32 = 30.0;

// open a few players, like spotify and a browser tab, and each gets its own row
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let media = Media::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for player in media.players() {
        let active = media.active() == Some(player);

        rows.push(Box::new(player_row(player, active)));
    }

    let height = ROW_HEIGHT * media.players().len().max(1) as f32;

    LayerWindow::new()
        .width(Full)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Column::new(rows))
}

// the active player, the one Media::play_pause() controls, is the lighter row
fn player_row(player: &MediaPlayer, active: bool) -> Row {
    let fill = if active { Color::BLUE } else { Color::BLACK };

    let symbol = if player.playing() { "pause" } else { "play" };

    let label = format!(
        "{}: {} - {}",
        player.identity(),
        player.artist(),
        player.title()
    );

    // each button gets its own copy, so it keeps controlling this player
    let previous = player.clone();
    let play_pause = player.clone();
    let next = player.clone();

    Row::new(children![
        button("prev").on_click(move |_| previous.previous()),
        button(symbol).on_click(move |_| play_pause.play_pause()),
        button("next").on_click(move |_| next.next()),
        Rectangle::new()
            .width(Parent)
            .height(ROW_HEIGHT)
            .fill(fill)
            .child(Text::new(label).size(18.0).color(Color::WHITE)),
    ])
}

fn button(label: &str) -> Rectangle {
    Rectangle::new()
        .width(80.0)
        .height(ROW_HEIGHT)
        .fill(Color::BLUE)
        .child(Text::new(label).size(18.0).color(Color::WHITE))
}
