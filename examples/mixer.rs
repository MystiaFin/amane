use amane::{
    App, Audio, AudioApp, Color, Column, Full, Layer, LayerWindow, Parent, Rectangle, Row, Scroll,
    Service, Text, Vertical, Widget, children,
};

const ROW_HEIGHT: f32 = 30.0;

// each wheel step moves an app's volume by this much
const STEP: i32 = 5;

// play sound in a few apps, like spotify and a browser tab, and each gets its own row
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let audio = Audio::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for app in audio.apps() {
        rows.push(Box::new(app_row(app)));
    }

    let height = ROW_HEIGHT * audio.apps().len().max(1) as f32;

    LayerWindow::new()
        .width(Full)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Column::new(rows))
}

// click mutes the app, scrolling changes its volume
fn app_row(app: &AudioApp) -> Row {
    let label = if app.muted() {
        format!("{}: muted", app.name())
    } else {
        format!("{}: {}%", app.name(), app.volume())
    };

    // each handler gets its own copy, so it keeps controlling this app
    let muted = app.clone();
    let scrolled = app.clone();

    Row::new(children![
        Rectangle::new()
            .width(Parent)
            .height(ROW_HEIGHT)
            .fill(Color::BLACK)
            .on_click(move |_| Audio::toggle_app_mute(&muted))
            .on_scroll(move |scroll| change_volume(&scrolled, scroll))
            .child(Text::new(label).size(18.0).color(Color::WHITE)),
    ])
}

// wheel up is a negative y, which turns the volume up
fn change_volume(app: &AudioApp, scroll: Scroll) {
    let step = if scroll.y < 0.0 { STEP } else { -STEP };

    let changed = (i32::from(app.volume()) + step).clamp(0, 100);

    Audio::set_app_volume(app, changed as u8);
}
