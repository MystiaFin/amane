use amane::{App, Center, Color, Full, LayerWindow, Rectangle, Stack, Text, children};

// a card with a badge laid over its corner
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let card = Rectangle::new()
        .width(200.0)
        .height(120.0)
        .radius(16.0)
        .fill("#1e1e2e")
        .align_child(Center, Center)
        .child(Text::new("inbox").size(20.0).color(Color::WHITE));

    // translate moves the badge from the stack's top-left corner to the card's top-right
    let badge = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(Full)
        .fill("#f38ba8")
        .translate(186.0, -14.0)
        .align_child(Center, Center)
        .child(Text::new("3").size(14.0).color(Color::BLACK));

    LayerWindow::new()
        .width(260.0)
        .height(180.0)
        .child(
            Rectangle::new()
                .width(260.0)
                .height(180.0)
                .align_child(Center, Center)
                .child(Stack::new(children![card, badge])),
        )
}
