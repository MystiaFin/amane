mod left;

use amane::{Full, Layer, LayerWindow, Monitor, Parent, Rectangle, Vertical, Zone};

pub fn view(monitor: &Monitor) -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(40.0)
        .space(Zone::Reserve)
        .layer(Layer::Top)
        .anchor_vertical(Vertical::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#0000FF")
                .child(left::view(monitor)),
        )
}
