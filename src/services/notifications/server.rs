use crate::{Argument, Bus, Method, Notification, Notifications, Service, Urgency};

use super::Reason;

const NAME: &str = "org.freedesktop.Notifications";

const PATH: &str = "/org/freedesktop/Notifications";

const INTERFACE: &str = "org.freedesktop.Notifications";

pub fn run() {
    let bus = Bus::session();

    // watched before owning the name, so no early call is missed
    let methods = bus.methods(PATH, INTERFACE);

    if !bus.own(NAME) {
        return;
    }

    Notifications::write().running = true;

    for method in methods {
        answer(&method);
    }
}

fn answer(method: &Method) {
    match method.name() {
        "Notify" => notify(method),
        "CloseNotification" => close(method),
        "GetCapabilities" => send_capabilities(method),
        "GetServerInformation" => send_information(method),

        // every call waits for an answer, even one amane does not know
        _ => method.reply(&[]),
    }
}

fn notify(method: &Method) {
    let [
        app_name,
        replaces_id,
        icon,
        summary,
        body,
        _actions,
        hints,
        _timeout,
    ] = method.arguments()
    else {
        method.reply(&[]);

        return;
    };

    let notification = Notification {
        // add() picks the real id
        id: 0,

        app_name: String::from(app_name.text()),
        summary: String::from(summary.text()),
        body: String::from(body.text()),
        icon: String::from(icon.text()),

        urgency: Urgency::from_level(hints.get("urgency").number()),
    };

    let replaces_id = replaces_id.number() as u32;

    let id = Notifications::write().add(notification, replaces_id);

    method.reply(&[Argument::from(id)]);
}

fn close(method: &Method) {
    let id = method.arguments().first().map(|id| id.number() as u32);

    if let Some(id) = id {
        Notifications::write().close(id, Reason::Closed);
    }

    method.reply(&[]);
}

// "body" is the only extra the spec lets a server claim without handling actions or images
fn send_capabilities(method: &Method) {
    let capabilities = vec![String::from("body")];

    method.reply(&[Argument::from(capabilities)]);
}

fn send_information(method: &Method) {
    let information = [
        Argument::from("amane"),
        Argument::from("amane"),
        Argument::from(env!("CARGO_PKG_VERSION")),
        Argument::from("1.2"),
    ];

    method.reply(&information);
}

pub fn announce_closed(id: u32, reason: Reason) {
    let arguments = [Argument::from(id), Argument::from(reason as u32)];

    Bus::session().emit(PATH, INTERFACE, "NotificationClosed", &arguments);
}
