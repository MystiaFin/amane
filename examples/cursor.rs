use amane::{
    App, Color, Grab, Keyboard, LayerWindow, NotAllowed, Parent, Pointer, Rectangle,
    ResizeHorizontal, Row, TextInput, children,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(600.0)
        .height(120.0)
        .keyboard(Keyboard::OnDemand)
        .child(Row::new(children![
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::RED)
                .cursor(Pointer),
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::GREEN)
                .cursor(Grab),
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .cursor(ResizeHorizontal),
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .cursor(NotAllowed),
            // a text input shows the text cursor on its own
            Rectangle::new()
                .width(150.0)
                .height(Parent)
                .fill(Color::WHITE)
                .child(TextInput::new("cursor").width(150.0))
        ]))
}
