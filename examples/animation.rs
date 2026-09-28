use std::time::Duration;

use amane::{
    Animation, App, Button, Color, Easing, Horizontal, Layer, LayerWindow, Margin, Parent,
    Rectangle, Service, Text, Vertical,
};

struct Panel {
    open: bool,
    tall: bool,

    // margin above the window, below 0 pushes it off the top of the screen
    slide: Animation,
    fade: Animation,

    height: Animation,
    color: Animation<Color>,
}

impl Service for Panel {
    fn new() -> Self {
        let slow = Duration::from_millis(400);

        Self {
            open: true,
            tall: false,

            slide: Animation::new(10.0).duration(slow),
            fade: Animation::new(1.0).duration(slow),

            height: Animation::new(100.0).easing(Easing::InOut),
            color: Animation::new(Color::BLUE),
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

    let opacity = panel.fade.value();

    let margin = Margin {
        top: panel.slide.value() as i32,
        right: 10,
        bottom: 0,
        left: 0,
    };

    // stays on screen until fading out has finished
    let visible = panel.open || opacity > 0.0;

    LayerWindow::new()
        .width(300.0)
        .height(panel.height.value())
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(margin)
        .layer(Layer::Top)
        .visible(visible)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(panel.color.value())
                .opacity(opacity)
                .on_click(panel_clicked)
                .child(Text::new("click to resize").size(20.0).color(Color::WHITE)),
        )
}

fn panel_clicked(_: Button) {
    let mut panel = Panel::write();

    panel.tall = !panel.tall;

    let (height, color) = if panel.tall {
        (400.0, Color::from("#8839ef"))
    } else {
        (100.0, Color::BLUE)
    };

    panel.height.to(height);
    panel.color.to(color);
}

// `amane ipc call toggle` slides the panel out of the top edge, and back in the next time
fn toggle(_: &[String]) -> String {
    let mut panel = Panel::write();

    panel.open = !panel.open;

    let (slide, fade) = if panel.open {
        (10.0, 1.0)
    } else {
        (-110.0, 0.0)
    };

    panel.slide.to(slide);
    panel.fade.to(fade);

    if panel.open {
        String::from("shown")
    } else {
        String::from("hidden")
    }
}
