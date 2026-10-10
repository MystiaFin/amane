use std::env;

use amane::{
    App, Color, Column, Pam, PamConfig, PamError, PamMessageKind, PamResult, Parent, Rectangle,
    Service, Text, TextInput, Widget, Window,
};

// A normal window: authenticating here never locks or unlocks the session.
fn main() {
    if let Err(error) = start() {
        eprintln!("{error}");
    }
    App::new().normal_window("pam", view).run();
}

fn start() -> Result<(), PamError> {
    let service = env::var("AMANE_PAM_SERVICE").unwrap_or_else(|_| String::from("login"));
    let mut config = PamConfig::new(&service);
    if let Some(directory) = env::var_os("AMANE_PAM_CONFIG_DIRECTORY") {
        config = config.config_directory(directory);
    }
    Pam::start(config)
}

fn view() -> Window {
    let pam = Pam::read();
    let status = match pam.result() {
        Some(PamResult::Success) => String::from("authenticated"),
        Some(PamResult::Rejected(_)) => String::from("authentication rejected"),
        Some(PamResult::Aborted) if pam.active() => String::from("cancelling..."),
        Some(PamResult::Aborted) => String::from("cancelled"),
        Some(PamResult::Error(error)) => error.to_string(),
        None if pam.active() => String::from("authenticating..."),
        None => String::from("authentication not started"),
    };
    let mut content: Vec<Box<dyn Widget>> =
        vec![Box::new(Text::new(status).size(24.0).color(Color::WHITE))];
    for message in pam.messages() {
        if matches!(message.kind(), PamMessageKind::Info | PamMessageKind::Error) {
            content.push(Box::new(Text::new(message.text()).color(Color::WHITE)));
        }
    }
    if let Some(prompt) = pam.prompt() {
        let id = prompt.id();
        content.push(Box::new(Text::new(prompt.text()).color(Color::WHITE)));
        let mut input = TextInput::new("pam-response")
            .focused()
            .on_submit(move |response| {
                if Pam::respond(id, &response).is_ok() {
                    TextInput::set_text("pam-response", "");
                }
            });
        if prompt.kind() == (PamMessageKind::Prompt { visible: false }) {
            input = input.password();
        }
        content.push(Box::new(
            Rectangle::new()
                .width(300.0)
                .height(40.0)
                .fill(Color::WHITE)
                .padding(8.0)
                .child(input),
        ));
    }
    content.push(Box::new(
        Rectangle::new()
            .width(300.0)
            .height(40.0)
            .fill("#cdd6f4")
            .padding(8.0)
            .child(Text::new(if pam.active() { "cancel" } else { "try again" }))
            .on_click(|_| {
                let active = Pam::read().active();
                if active {
                    Pam::abort();
                } else if let Err(error) = start() {
                    eprintln!("{error}");
                }
                TextInput::set_text("pam-response", "");
            }),
    ));
    Window::new()
        .title("PAM authentication")
        .size(400.0, 300.0)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#1e1e2e")
                .padding(24.0)
                .child(Column::new(content).gap(12.0)),
        )
}
