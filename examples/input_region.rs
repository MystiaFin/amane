use amane::{App, Color, InputArea, LayerWindow, Parent, Rectangle, Row, children};

fn main() {
    App::new().window(view).window(overlay).run();
}

// only the red square takes the pointer, clicks on the blue part reach the windows below
fn view() -> LayerWindow {
    LayerWindow::new()
        .width(400.0)
        .height(100.0)
        .input_region(vec![InputArea {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        }])
        .child(
            Row::new(children![
                Rectangle::new().width(100.0).height(Parent).fill(Color::RED),
                Rectangle::new().width(Parent).height(Parent).fill(Color::BLUE),
            ])
            .width(Parent)
            .height(Parent),
        )
}

// a window that never takes the pointer at all
fn overlay() -> LayerWindow {
    LayerWindow::new()
        .width(200.0)
        .height(200.0)
        .click_through()
        .child(Rectangle::new().width(Parent).height(Parent).fill(Color::GREEN))
}
