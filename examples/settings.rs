use amane::{
    App, Center, Color, Full, LayerWindow, Parent, Pointer, Rectangle, Text, Vertical, Window,
    spawn,
};

// a bar on top, and a normal window beside it like a settings app would have
fn main() {
    App::new()
        .window(bar)
        .normal_window("settings", settings)
        .run();
}

fn bar() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(32.0)
        .anchor_vertical(Vertical::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#1e1e2e")
                .cursor(Pointer)
                .on_click(|_| spawn("notify-send amane 'the bar was clicked'")),
        )
}

fn settings() -> Window {
    Window::new()
        .title("amane settings")
        .size(480.0, 320.0)
        .resizable(true)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#313244")
                .align_child(Center, Center)
                .child(Text::new("settings go here").size(20.0).color(Color::WHITE)),
        )
}
