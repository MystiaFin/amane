use std::time::Duration;

use amane::{
    Animation, App, Button, Color, Column, Easing, LayerWindow, Rectangle, Service, Spring,
    children,
};

// click to send both boxes to the other side, click again halfway to see the spring turn smoothly
struct Boxes {
    right: bool,

    spring: Spring,
    curve: Animation,
}

impl Service for Boxes {
    fn new() -> Self {
        let curve = Animation::new(0.0)
            .duration(Duration::from_millis(340))
            .easing(Easing::Curve(0.2, 0.0, 0.0, 1.0));

        Self {
            right: false,
            spring: Spring::new(0.0).precision(0.1),
            curve,
        }
    }

    // nothing changes on its own, only through clicks
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let boxes = Boxes::read();

    let spring_box = Rectangle::new()
        .width(60.0)
        .height(60.0)
        .radius(12.0)
        .fill(Color::from("#89b4fa"))
        .translate(boxes.spring.value(), 0.0);

    let curve_box = Rectangle::new()
        .width(60.0)
        .height(60.0)
        .radius(12.0)
        .fill(Color::from("#f5c2e7"))
        .translate(boxes.curve.value(), 0.0);

    LayerWindow::new().width(400.0).height(140.0).child(
        Rectangle::new()
            .width(400.0)
            .height(140.0)
            .fill("#1e1e2e")
            .on_click(clicked)
            .child(Column::new(children![spring_box, curve_box]).gap(10.0)),
    )
}

fn clicked(_: Button) {
    let mut boxes = Boxes::write();

    boxes.right = !boxes.right;

    let target = if boxes.right { 340.0 } else { 0.0 };

    boxes.spring.to(target);
    boxes.curve.to(target);
}
