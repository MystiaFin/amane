use amane::{
    App, Audio, Button, Color, Full, Layer, LayerWindow, Parent, Rectangle, Row, Scroll, Service,
    Text, Vertical, children,
};

// scroll over the bar to change the microphone level, click to mute it
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let audio = Audio::read();

    let label = if audio.microphone_muted() {
        String::from("microphone muted")
    } else {
        format!("microphone {}%", audio.microphone_volume())
    };

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Row::new(children![
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .on_click(clicked)
                .on_scroll(scrolled)
                .child(Text::new(label).size(20.0).color(Color::WHITE)),
        ]))
}

fn clicked(_: Button) {
    Audio::toggle_microphone_mute();
}

// each wheel step moves the level by 5
fn scrolled(scroll: Scroll) {
    let volume = i32::from(Audio::read().microphone_volume());

    let step = if scroll.y < 0.0 { 5 } else { -5 };

    let changed = (volume + step).clamp(0, 100);

    Audio::set_microphone_volume(changed as u8);
}
