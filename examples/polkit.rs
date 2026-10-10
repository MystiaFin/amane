use std::cell::Cell;

use amane::{
    App, Color, Column, Keyboard, Layer, LayerWindow, Parent, Polkit, PolkitConfig, Rectangle, Row,
    Service, Text, TextInput, Widget, Zone,
};

thread_local! {
    static PROMPT: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
}

fn main() {
    if let Err(error) = Polkit::start(PolkitConfig::new()) {
        eprintln!("{error}");
    }
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let polkit = Polkit::read();
    let prompt = polkit
        .flow()
        .and_then(|flow| flow.prompt().map(|prompt| (flow.id(), prompt.id())));
    // Keep partially typed credentials out of later prompts, including after daemon cancellation.
    PROMPT.with(|previous| {
        if previous.replace(prompt) != prompt {
            TextInput::set_text("polkit-response", "");
        }
    });
    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    if let Some(error) = polkit.error() {
        children.push(Box::new(
            Text::new(error.to_string()).color(Color::from("#f38ba8")),
        ));
        children.push(button("retry registration", || {
            let _ = Polkit::start(PolkitConfig::new());
        }));
    }
    if let Some(flow) = polkit.flow() {
        let request = flow.id();
        children.push(Box::new(
            Text::new(flow.message())
                .size(22.0)
                .color(Color::from("#cdd6f4")),
        ));
        let mut identities = Vec::new();
        for (index, identity) in flow.identities().iter().enumerate() {
            let label = if index == flow.selected_identity() {
                format!("{} (selected)", identity.name())
            } else {
                identity.name().to_string()
            };
            identities.push(button(&label, move || {
                let _ = Polkit::select_identity(request, index);
                TextInput::set_text("polkit-response", "");
            }));
        }
        children.push(Box::new(Row::new(identities).gap(8.0)));
        if let Some(message) = flow.supplementary() {
            children.push(Box::new(Text::new(message.text()).color(Color::from(
                if message.is_error() {
                    "#f38ba8"
                } else {
                    "#cdd6f4"
                },
            ))));
        }
        if let Some(prompt) = flow.prompt() {
            let prompt_id = prompt.id();
            children.push(Box::new(
                Text::new(prompt.text()).color(Color::from("#cdd6f4")),
            ));
            let mut input =
                TextInput::new("polkit-response")
                    .focused()
                    .on_submit(move |response| {
                        let _ = Polkit::respond(request, prompt_id, &response);
                        TextInput::set_text("polkit-response", "");
                    });
            if !prompt.visible() {
                input = input.password();
            }
            children.push(Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(40.0)
                    .fill("#cdd6f4")
                    .padding(8.0)
                    .child(input),
            ));
        } else if flow.can_retry() {
            children.push(button("try again", move || {
                let _ = Polkit::retry(request);
            }));
        }
        children.push(button("cancel", move || {
            let _ = Polkit::cancel(request);
            TextInput::set_text("polkit-response", "");
        }));
    }
    LayerWindow::new()
        .width(520.0)
        .height(360.0)
        .namespace("amane-polkit")
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(Keyboard::Exclusive)
        .visible(polkit.flow().is_some() || polkit.error().is_some())
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#1e1e2e")
                .padding(24.0)
                .child(Column::new(children).gap(12.0)),
        )
}

fn button(label: &str, clicked: impl Fn() + 'static) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(150.0)
            .height(40.0)
            .fill("#cdd6f4")
            .padding(8.0)
            .child(Text::new(label))
            .on_click(move |_| clicked()),
    )
}
