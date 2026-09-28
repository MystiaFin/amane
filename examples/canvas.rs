use amane::{
    App, Arc, Canvas, Cap, Circle, LayerWindow, Line, Parent, Path, Rectangle, Row, Shape,
    children, shapes,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(720.0).height(200.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill("#1e1e2e")
            .child(Row::new(children![
                gap(),
                ring(0.7),
                gap(),
                wave(0.6, 0.0),
                spirograph(),
                gauge(0.4),
                gap(),
            ])),
    )
}

fn gap() -> Rectangle {
    Rectangle::new().width(20.0).height(200.0)
}

// a progress ring, value goes from 0 to 1
fn ring(value: f32) -> Canvas {
    Canvas::new().width(120.0).height(200.0).shapes(shapes![
        Arc::new().stroke(10.0, "#45475a"),
        Arc::new()
            .sweep(360.0 * value)
            .stroke(10.0, "#89b4fa")
            .cap(Cap::Round),
        Circle::new().radius(40.0).fill("#89b4fa").opacity(0.3),
    ])
}

// a wavy progress bar, moving phase over time makes the wave flow
fn wave(progress: f32, phase: f32) -> Canvas {
    let width = 200.0;
    let middle = 100.0;
    let end = width * progress;

    let mut line = Path::new().move_to(0.0, middle);

    for step in 0..=end as u32 {
        let x = step as f32;
        let y = middle + f32::sin(x / 8.0 + phase) * 8.0;

        line = line.line_to(x, y);
    }

    Canvas::new().width(width).height(200.0).shapes(shapes![
        line.stroke(4.0, "#a6e3a1").cap(Cap::Round),
        Line::new()
            .from(end + 8.0, middle)
            .to(width, middle)
            .stroke(4.0, "#45475a")
            .cap(Cap::Round),
    ])
}

// a small wheel rolling inside a big one, traced by a pen fixed to the small wheel
fn spirograph() -> Canvas {
    let center = 100.0;

    let big = 60.0;
    let small = 22.0;
    let pen = 30.0;
    let spin = (big - small) / small;

    let mut line = Path::new().move_to(center + big - small + pen, center);

    for step in 1..=7000 {
        let turn = step as f32 / 100.0;

        let x = center + (big - small) * f32::cos(turn) + pen * f32::cos(spin * turn);
        let y = center + (big - small) * f32::sin(turn) - pen * f32::sin(spin * turn);

        line = line.line_to(x, y);
    }

    Canvas::new()
        .width(200.0)
        .height(200.0)
        .shapes(shapes![line.stroke(1.5, "#cba6f7").opacity(0.8)])
}

// a half circle gauge with a needle
fn gauge(value: f32) -> Canvas {
    let angle = f32::to_radians(-90.0 + 180.0 * value);

    let needle_x = 60.0 + f32::sin(angle) * 36.0;
    let needle_y = 120.0 - f32::cos(angle) * 36.0;

    Canvas::new().width(120.0).height(200.0).shapes(shapes![
        Arc::new()
            .center(60.0, 120.0)
            .radius(48.0)
            .start(-90.0)
            .sweep(180.0)
            .stroke(8.0, "#45475a")
            .cap(Cap::Round),
        Arc::new()
            .center(60.0, 120.0)
            .radius(48.0)
            .start(-90.0)
            .sweep(180.0 * value)
            .stroke(8.0, "#f9e2af")
            .cap(Cap::Round),
        Line::new()
            .from(60.0, 120.0)
            .to(needle_x, needle_y)
            .stroke(3.0, "#cdd6f4")
            .cap(Cap::Round),
        Circle::new()
            .center(60.0, 120.0)
            .radius(6.0)
            .fill("#cdd6f4"),
    ])
}
