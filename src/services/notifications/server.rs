use std::time::SystemTime;

use crate::{Argument, Bus, Method, Notification, Notifications, Service, Urgency, Value};

use super::{Action, Reason, image};

const NAME: &str = "org.freedesktop.Notifications";

const PATH: &str = "/org/freedesktop/Notifications";

const INTERFACE: &str = "org.freedesktop.Notifications";

// the key of what clicking the notification itself does
pub const DEFAULT_ACTION: &str = "default";

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
        actions,
        hints,
        _timeout,
    ] = method.arguments()
    else {
        method.reply(&[]);

        return;
    };

    let (actions, has_default) = read_actions(actions);

    let notification = Notification {
        // add() picks the real id
        id: 0,

        app_name: String::from(app_name.text()),
        summary: String::from(summary.text()),
        body: String::from(body.text()),
        icon: String::from(icon.text()),
        image: image::read(hints),

        urgency: Urgency::from_level(hints.get("urgency").number()),

        actions,
        has_default,

        resident: hints.get("resident").bool(),

        received: SystemTime::now(),
    };

    let replaces_id = replaces_id.number() as u32;

    let id = Notifications::write().add(notification, replaces_id);

    method.reply(&[Argument::from(id)]);
}

/*
 * actions come as one flat list: key, label, key, label;
 * the "default" one is what clicking the notification itself does
 */
fn read_actions(list: &Value) -> (Vec<Action>, bool) {
    let mut actions = Vec::new();
    let mut has_default = false;

    for pair in list.list().chunks_exact(2) {
        let key = pair[0].text();
        let label = pair[1].text();

        if key == DEFAULT_ACTION {
            has_default = true;

            continue;
        }

        actions.push(Action {
            key: String::from(key),
            label: String::from(label),
        });
    }

    (actions, has_default)
}

fn close(method: &Method) {
    let id = method.arguments().first().map(|id| id.number() as u32);

    if let Some(id) = id {
        Notifications::write().close(id, Reason::Closed);
    }

    method.reply(&[]);
}

fn send_capabilities(method: &Method) {
    let capabilities = vec![
        String::from("actions"),
        String::from("body"),
        String::from("icon-static"),
    ];

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

// the sender hears which button was pressed, and does the rest itself
pub fn announce_action(id: u32, key: &str) {
    let arguments = [Argument::from(id), Argument::from(key)];

    Bus::session().emit(PATH, INTERFACE, "ActionInvoked", &arguments);
}

pub fn announce_closed(id: u32, reason: Reason) {
    let arguments = [Argument::from(id), Argument::from(reason as u32)];

    Bus::session().emit(PATH, INTERFACE, "NotificationClosed", &arguments);
}
