use amane::{App, Column, LayerWindow, Mask, Parent, Rectangle, Row, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(520.0).height(160.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .radius(24.0)
            .fill("#1e1e2e")
            .child(Column::new(children![
                Rectangle::new().width(Parent).height(40.0),
                Row::new(children![
                    Rectangle::new().width(40.0).height(80.0),
                    // a round hole straight through to what is behind the surface
                    Rectangle::new()
                        .width(80.0)
                        .height(80.0)
                        .radius(40.0)
                        .fill(Mask),
                    Rectangle::new().width(40.0).height(80.0),
                    // a rounded hole with an outline
                    Rectangle::new()
                        .width(120.0)
                        .height(80.0)
                        .radius(16.0)
                        .fill(Mask)
                        .border(2.0, "#cdd6f4"),
                    Rectangle::new().width(40.0).height(80.0),
                    // only cuts the card holding it, so the dark background shows in the hole
                    Rectangle::new()
                        .width(120.0)
                        .height(80.0)
                        .radius(16.0)
                        .fill("#89b4fa")
                        .child(Column::new(children![
                            Rectangle::new().width(Parent).height(20.0),
                            Row::new(children![
                                Rectangle::new().width(20.0).height(40.0),
                                Rectangle::new()
                                    .width(80.0)
                                    .height(40.0)
                                    .radius(20.0)
                                    .fill(Mask),
                            ]),
                        ])),
                ]),
            ])),
    )
}
