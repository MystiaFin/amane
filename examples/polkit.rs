use std::cell::Cell;

use amane::{
    App, Color, Column, Key, Keyboard, Layer, LayerWindow, Parent, Pointer, Polkit, PolkitConfig,
    Rectangle, Row, Service, SpaceBetween, Text, TextInput, Widget, Zone,
};

thread_local! {
    static PROMPT: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
}

fn main() {
    if let Err(error) = Polkit::start(PolkitConfig::new()) {
        eprintln!("{error}");
    }
    App::new().without_ipc().window(view).run();
    let _ = Polkit::stop();
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
            Column::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("Unable to start Polkit agent")
                                .size(22.0)
                                .color(Color::from("#cdd6f4")),
                        ),
                        Box::new(
                            Text::new(error.to_string())
                                .wrap()
                                .max_lines(6)
                                .color(Color::from("#f38ba8")),
                        ),
                        Box::new(
                            Text::new("Resolve the error before retrying. Only one agent can run per login session.")
                                .wrap()
                                .max_lines(2)
                                .color(Color::from("#cdd6f4")),
                        ),
                    ])
                    .gap(12.0),
                ),
                Box::new(
                    Row::new(vec![
                        button("retry registration", || {
                            let _ = Polkit::start(PolkitConfig::new());
                        }),
                        button("quit", App::quit),
                    ])
                    .gap(8.0),
                ),
            ])
            .height(Parent)
            .justify(SpaceBetween),
        ));
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
    let request = polkit.flow().map(|flow| flow.id());
    LayerWindow::new()
        .width(520.0)
        .height(360.0)
        .namespace("amane-polkit")
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(if polkit.error().is_some() {
            Keyboard::OnDemand
        } else {
            Keyboard::Exclusive
        })
        .visible(polkit.flow().is_some() || polkit.error().is_some())
        .on_key(move |key| {
            if key == Key::Escape {
                if let Some(request) = request {
                    let _ = Polkit::cancel(request);
                } else {
                    App::quit();
                }
                TextInput::set_text("polkit-response", "");
            }
        })
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
            .cursor(Pointer)
            .padding(8.0)
            .child(Text::new(label))
            .on_click(move |_| clicked()),
    )
}
