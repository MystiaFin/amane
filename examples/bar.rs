use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical, Zone};

fn main() {
    App::new()
        .window(
            LayerWindow::new()
                .width(Full)
                .height(30.0)
                .anchor_vertical(Vertical::Top)
                .layer(Layer::Top)
                .space(Zone::Reserve)
                .child(
                    Rectangle::new()
                        .width(Parent)
                        .height(Parent)
                        .color(Color::BLUE)
                        .child(Text::new("Amane bar").size(20.0).color(Color::WHITE)),
                ),
        )
        .run();
}
