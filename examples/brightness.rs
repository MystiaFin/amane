use amane::{
    App, Brightness, Color, Full, Layer, LayerWindow, Parent, Rectangle, Row, Scroll, Service, Text,
    Vertical, children,
};

// scroll over the bar to change the screen brightness
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let brightness = Brightness::read();

    let label = if brightness.present() {
        format!("brightness {}%", brightness.percent())
    } else {
        String::from("no backlight")
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
                .on_scroll(scrolled)
                .child(Text::new(label).size(20.0).color(Color::WHITE)),
        ]))
}

// each wheel step moves the brightness by 5
fn scrolled(scroll: Scroll) {
    let brightness = i32::from(Brightness::read().percent());

    let step = if scroll.y < 0.0 { 5 } else { -5 };

    let changed = (brightness + step).clamp(1, 100);

    Brightness::set(changed as u8);
}
