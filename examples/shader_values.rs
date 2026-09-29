use std::time::Duration;

use amane::{App, Button, LayerWindow, Rectangle, Service};

// click anywhere to add a spot, the shader draws every spot it is handed
fn main() {
    App::new().window(view).run();
}

struct Spots {
    list: Vec<[f32; 4]>,
}

impl Service for Spots {
    fn new() -> Self {
        let list = vec![[60.0, 100.0, 30.0, 0.0], [200.0, 80.0, 50.0, 0.0]];

        Self { list }
    }

    // nothing changes on its own, only through clicks
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

// run from the project folder, the shader path is relative to it
fn view() -> LayerWindow {
    let spots = Spots::read();

    let count = spots.list.len() as f32;

    // the first row says how many spots follow
    let mut values = vec![[count, 0.0, 0.0, 0.0]];

    values.extend(&spots.list);

    LayerWindow::new().width(420.0).height(200.0).child(
        Rectangle::new()
            .width(420.0)
            .height(200.0)
            .fill("#1e1e2e")
            .shader("examples/shaders/spots.wgsl")
            .shader_values(values)
            .on_click(clicked),
    )
}

// one row is taken by the count, so 15 spots fit
fn clicked(_: Button) {
    let mut spots = Spots::write();

    if spots.list.len() >= 15 {
        spots.list.remove(0);
    }

    let offset = spots.list.len() as f32 * 25.0;

    spots.list.push([20.0 + offset, 150.0, 15.0, 0.0]);
}
