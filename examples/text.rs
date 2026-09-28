use amane::{App, Column, LayerWindow, Text, Weight, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let long = "the quick brown fox jumps over the lazy dog, then keeps running far past the edge";

    LayerWindow::new()
        .width(260.0)
        .height(400.0)
        .child(Column::new(children![
            Text::new("regular"),
            Text::new("bold").weight(Weight::Bold),
            Text::new("light, picked by number").weight(300),
            Text::new(long).elide(),
            Text::new(long).wrap().max_lines(2).elide(),
            Text::new(long).wrap()
        ]))
}
