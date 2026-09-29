use std::f32::consts::TAU;
use std::sync::LazyLock;
use std::time::Instant;

use amane::{App, Color, Full, LayerWindow, Rectangle};

// a dot that swings back and forth forever, moved by the example itself instead of an Animation
static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let seconds = START.elapsed().as_secs_f32();

    // one full swing every two seconds
    let swing = (seconds * TAU / 2.0).sin();

    let x = 170.0 + swing * 150.0;

    // it never arrives, so it always wants the next frame
    amane::request_frame();

    LayerWindow::new().width(400.0).height(60.0).child(
        Rectangle::new()
            .width(400.0)
            .height(60.0)
            .fill("#1e1e2e")
            .child(
                Rectangle::new()
                    .width(60.0)
                    .height(60.0)
                    .radius(Full)
                    .fill(Color::from("#89b4fa"))
                    .translate(x, 0.0),
            ),
    )
}
