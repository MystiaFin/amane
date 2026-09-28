use amane::{
    App, Battery, Color, Cpu, Full, Layer, LayerWindow, Memory, Parent, Rectangle, Service, Text,
    Vertical,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let battery = Battery::read();
    let memory = Memory::read();
    let cpu = Cpu::read();

    let battery_label = if !battery.present() {
        String::from("no battery")
    } else if battery.charging() {
        format!("battery {}% charging", battery.percent())
    } else {
        format!("battery {}%", battery.percent())
    };

    let label = format!(
        "{battery_label}   memory {}%   cpu {}%",
        memory.percent(),
        cpu.percent()
    );

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLACK)
                .child(Text::new(&label).size(16.0).color(Color::WHITE)),
        )
}
