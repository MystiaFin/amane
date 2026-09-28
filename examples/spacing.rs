use amane::{
    App, Center, Color, Column, End, LayerWindow, Padding, Parent, Rectangle, Row, children,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(400.0).height(400.0).child(
        Column::new(children![
            // three squares with 12 pixels between each
            Row::new(children![
                Rectangle::new().width(60.0).height(60.0).fill(Color::RED),
                Rectangle::new().width(60.0).height(60.0).fill(Color::GREEN),
                Rectangle::new().width(60.0).height(60.0).fill(Color::BLUE),
            ])
            .gap(12.0),
            // the same space on every side
            Rectangle::new()
                .width(Parent)
                .height(100.0)
                .fill(Color::GREEN)
                .padding(16.0)
                .child(Rectangle::new().width(Parent).height(Parent).fill(Color::BLUE)),
            // a different space per side, with the child in the bottom right corner
            Rectangle::new()
                .width(Parent)
                .height(100.0)
                .fill(Color::RED)
                .padding(Padding {
                    top: 4.0,
                    right: 24.0,
                    bottom: 8.0,
                    left: 4.0,
                })
                .align_child(End, End)
                .child(Rectangle::new().width(40.0).height(40.0).fill(Color::BLUE)),
            // a child in the middle
            Rectangle::new()
                .width(Parent)
                .height(80.0)
                .fill(Color::BLUE)
                .align_child(Center, Center)
                .child(Rectangle::new().width(40.0).height(40.0).fill(Color::RED)),
        ])
        .width(Parent)
        .height(Parent)
        .gap(16.0),
    )
}
