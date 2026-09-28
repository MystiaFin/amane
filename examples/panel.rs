use std::time::Duration;

use amane::{
    App, Button, Color, Horizontal, Layer, LayerWindow, Margin, Parent, Rectangle, Service, Text,
    Vertical,
};

struct Panel {
    open: bool,
    tall: bool,
}

impl Service for Panel {
    fn new() -> Self {
        Self {
            open: true,
            tall: false,
        }
    }

    // nothing changes on its own, only through input and ipc
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }

    fn update(&mut self) {}
}

fn main() {
    App::new().ipc("toggle", toggle).window(view).run();
}

fn view() -> LayerWindow {
    let panel = Panel::read();

    let height = if panel.tall { 400.0 } else { 100.0 };

    let margin = Margin {
        top: 10,
        right: 10,
        bottom: 0,
        left: 0,
    };

    LayerWindow::new()
        .width(300.0)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(margin)
        .layer(Layer::Top)
        .visible(panel.open)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .on_click(panel_clicked)
                .child(Text::new("click to resize").size(20.0).color(Color::WHITE)),
        )
}

fn panel_clicked(_: Button) {
    let mut panel = Panel::write();

    panel.tall = !panel.tall;
}

// `amane ipc call toggle` hides the panel, and shows it again the next time
fn toggle(_: &[String]) -> String {
    let mut panel = Panel::write();

    panel.open = !panel.open;

    if panel.open {
        String::from("shown")
    } else {
        String::from("hidden")
    }
}
